import { useCallback, useEffect, useState } from "react";
import {
  ShieldCheck,
  KeyRound,
  CreditCard,
  QrCode,
  MonitorSmartphone,
  ShieldAlert,
  Radio,
  FileText,
  Download,
  Check,
  Copy,
  RefreshCw,
  Sparkles,
  Lock,
  Plus,
  ArrowRight,
  CheckCircle2,
  Building2,
} from "lucide-react";
import { rpc, type VendorDashboardOverview } from "@/gen/rpc";

export type VendorTab =
  "customers" | "studio" | "installations" | "revocations" | "releases" | "audit";

export interface VendorAdminViewProps {
  onLicenseChanged?: () => void;
  initialOverview?: VendorDashboardOverview | null;
  defaultTab?: VendorTab;
}

export function VendorAdminView({
  onLicenseChanged,
  initialOverview = null,
  defaultTab = "customers",
}: VendorAdminViewProps) {
  const [overview, setOverview] = useState<VendorDashboardOverview | null>(
    initialOverview
  );
  const [loading, setLoading] = useState(initialOverview === null);
  const [activeTab, setActiveTab] = useState<VendorTab>(defaultTab);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [successBanner, setSuccessBanner] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Payment Form State
  const [payCustomerId, setPayCustomerId] = useState("cust_mirpur");
  const [paySubId, setPaySubId] = useState("sub_mirpur");
  const [payMethod, setPayMethod] = useState("bkash");
  const [payAmountBdt, setPayAmountBdt] = useState(13500);
  const [payTxRef, setPayTxRef] = useState("BKASH-TRX-994821");
  const [payPeriodMonths, setPayPeriodMonths] = useState(6);
  const [latestPairingCode, setLatestPairingCode] = useState<string | null>(null);
  const [latestSignedBlob, setLatestSignedBlob] = useState<{
    filename: string;
    blob: string;
  } | null>(null);

  // New Customer State
  const [newOrgName, setNewOrgName] = useState("");
  const [newOwnerName, setNewOwnerName] = useState("");
  const [newPhone, setNewPhone] = useState("");
  const newDistrict = "Dhaka";
  const [newPlanCode, setNewPlanCode] = useState("local_pro");

  // Offline Studio State
  const [offlineLicenseId, setOfflineLicenseId] = useState("LIC-2026-MIRP-0001");
  const [offlineRequestCode, setOfflineRequestCode] = useState("");
  const offlineReason = "Offline desktop activation via WhatsApp request code";

  // Revocation State
  const [revokeLicenseId, setRevokeLicenseId] = useState("LIC-2026-CTGS-0002");
  const [revokeReason, setRevokeReason] = useState(
    "Disputed bKash transaction / chargeback"
  );
  const [stepUpTotp, setStepUpTotp] = useState("");

  // Release Promotion & Key Recovery Drill State (Features 65 & 66, Spec 0042)
  const [promoteVersion, setPromoteVersion] = useState("1.0.0");
  const [promoteSourceChannel, setPromoteSourceChannel] = useState("internal");
  const [promoteTargetChannel, setPromoteTargetChannel] = useState<"beta" | "stable">(
    "beta"
  );
  const [promoteReason, setPromoteReason] = useState(
    "Zero-rebuild release promotion to test channel"
  );
  const [drillKeyType, setDrillKeyType] = useState<
    "windows_authenticode" | "tauri_updater_ed25519" | "license_ed25519"
  >("windows_authenticode");
  const [drillKeyId, setDrillKeyId] = useState("auth2026a");
  const [drillPrimaryCustodian, setDrillPrimaryCustodian] = useState("lead-sec-ops");
  const [drillSecondaryCustodian, setDrillSecondaryCustodian] = useState("cto-custodian");
  const [drillTestSha, setDrillTestSha] = useState(
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  );
  const [drillNotes, setDrillNotes] = useState(
    "Quarterly simulated hardware disaster recovery drill"
  );

  const loadOverview = useCallback(async () => {
    setLoading(true);
    const res = await rpc("vendor.get_overview", {});
    if (res.ok && res.data) {
      setOverview(res.data);
      if (!stepUpTotp && res.data.demo_totp_code) {
        setStepUpTotp(res.data.demo_totp_code);
      }
      if (res.data.customers.length > 0 && !paySubId) {
        setPayCustomerId(res.data.customers[0].id);
        setPaySubId(res.data.customers[0].subscription_id);
        setOfflineLicenseId(res.data.customers[0].license_id);
      }
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
    setLoading(false);
  }, [paySubId, stepUpTotp]);

  useEffect(() => {
    void loadOverview();
  }, [loadOverview]);

  const handleCopy = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch {
      // Clipboard fallback ignored
    }
  };

  const handleDownloadBlob = (filename: string, content: string) => {
    const blob = new Blob([content], {
      type: "application/json;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleRecordPayment = async () => {
    setErrorMessage(null);
    setSuccessBanner(null);
    const res = await rpc("vendor.record_payment", {
      admin_email: "ops@coachify.app",
      customer_id: payCustomerId,
      subscription_id: paySubId,
      amount_poisha: Math.round(Number(payAmountBdt) * 100),
      method: payMethod,
      reference: payTxRef,
      proof_url: null,
      period_months: Number(payPeriodMonths),
      installation_id: null,
    });
    if (res.ok && res.data) {
      setLatestPairingCode(res.data.pairing_code);
      setLatestSignedBlob({
        filename: `${res.data.license_id}-serial-${res.data.new_serial}.coachifylicense`,
        blob: res.data.signed_license_blob,
      });
      setSuccessBanner(
        `Invoice ${res.data.invoice_number} recorded! Subscription extended to ${res.data.new_paid_through} (Serial #${res.data.new_serial}). 6-Digit Pairing Code: ${res.data.pairing_code}`
      );
      setPayTxRef(`BKASH-TRX-${Math.floor(100000 + Math.random() * 900000)}`);
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleActivateLocalDesktopWithPairing = async (codeOrToken: string) => {
    setErrorMessage(null);
    const res = await rpc("license.activate_online", {
      code_or_token: codeOrToken,
      confirm_transfer: true,
      service_base_url: null,
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `Desktop activated online (${res.data.status})! Tier: ${res.data.license_status.tier}, Paid Through: ${res.data.license_status.paid_through}`
      );
      onLicenseChanged?.();
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleFillThisPcRequestCode = async () => {
    const res = await rpc("license.get_request_code", {});
    if (res.ok && res.data) {
      setOfflineRequestCode(res.data.qr_payload);
    }
  };

  const handleIssueOfflineLicense = async () => {
    setErrorMessage(null);
    setSuccessBanner(null);
    const res = await rpc("vendor.issue_offline_license", {
      admin_email: "ops@coachify.app",
      license_id: offlineLicenseId,
      request_code_or_qr: offlineRequestCode,
      reason: offlineReason,
    });
    if (res.ok && res.data) {
      setLatestSignedBlob({
        filename: res.data.download_filename,
        blob: res.data.signed_license_blob,
      });
      setSuccessBanner(
        `Offline license ${res.data.download_filename} signed for hardware ID ${res.data.installation_id} (Serial #${res.data.new_serial}).`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleCreateCustomer = async () => {
    if (!newOrgName.trim() || !newOwnerName.trim()) return;
    setErrorMessage(null);
    const res = await rpc("vendor.create_customer", {
      admin_email: "ops@coachify.app",
      org_name: newOrgName.trim(),
      owner_name: newOwnerName.trim(),
      phone: newPhone.trim() || "+8801700000000",
      email: null,
      district: newDistrict.trim() || "Dhaka",
      plan_code: newPlanCode,
    });
    if (res.ok && res.data) {
      setNewOrgName("");
      setNewOwnerName("");
      setNewPhone("");
      setSuccessBanner(
        `Created Coaching center "${res.data.org_name}" with License ${res.data.license_id}.`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleRevokeLicense = async () => {
    setErrorMessage(null);
    setSuccessBanner(null);
    const res = await rpc("vendor.revoke_license", {
      admin_email: "owner@coachify.app",
      license_id: revokeLicenseId,
      reason: revokeReason,
      step_up_totp_code: stepUpTotp || overview?.demo_totp_code || "",
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `License ${res.data.license_id} cryptographically revoked (ID: ${res.data.id}). Next desktop refresh will enforce read-only lock.`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleResetTransferCount = async (licenseId: string) => {
    setErrorMessage(null);
    const res = await rpc("vendor.reset_transfer_count", {
      license_id: licenseId,
      totp_code: stepUpTotp || overview?.demo_totp_code || "",
      admin_email: "ops@coachify.app",
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `Reset 12-month self-service transfer counter for ${res.data.license_id}.`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleUpdateReleaseStatus = async (
    version: string,
    channel: string,
    status: string
  ) => {
    setErrorMessage(null);
    const res = await rpc("vendor.update_release_status", {
      admin_email: "owner@coachify.app",
      version,
      channel,
      status,
      notes: null,
      step_up_totp_code: stepUpTotp || overview?.demo_totp_code || "",
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `Updated release ${res.data.version} (${res.data.channel}) status to ${res.data.status.toUpperCase()}.`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handlePromoteRelease = async () => {
    setErrorMessage(null);
    const res = await rpc("vendor.promote_release", {
      admin_email: "owner@coachify.app",
      version: promoteVersion,
      source_channel: promoteSourceChannel,
      target_channel: promoteTargetChannel,
      promotion_reason: promoteReason,
      step_up_totp_code: stepUpTotp || overview?.demo_totp_code || "",
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `Zero-rebuild promotion successful! Release v${res.data.version} is now HEAD of "${res.data.channel}" channel (SHA256: ${res.data.sha256.slice(0, 16)}...).`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  const handleRecordKeyRecoveryDrill = async () => {
    setErrorMessage(null);
    const res = await rpc("vendor.record_key_recovery_drill", {
      admin_email: "owner@coachify.app",
      key_type: drillKeyType,
      key_id: drillKeyId,
      custodian_primary: drillPrimaryCustodian,
      custodian_secondary: drillSecondaryCustodian,
      test_artifact_sha256: drillTestSha,
      notes: drillNotes,
      step_up_totp_code: stepUpTotp || overview?.demo_totp_code || "",
    });
    if (res.ok && res.data) {
      setSuccessBanner(
        `Key recovery drill recorded and cryptographically verified for "${res.data.key_type}" (${res.data.key_id}). Status: ${res.data.status}.`
      );
      await loadOverview();
    } else if (res.error) {
      setErrorMessage(res.error.message);
    }
  };

  return (
    <div
      className="rounded-[24px] border border-[#16110d]/12 bg-[#faf7f2] text-[#16110d] shadow-[0_24px_60px_-15px_rgba(22,17,13,0.12)] overflow-hidden"
      data-testid="vendor-admin-studio"
    >
      {/* Landing-Page WindowFrame Top Chrome */}
      <div className="bg-[#16110d] text-[#faf7f2] px-6 py-4 flex flex-wrap items-center justify-between gap-4 border-b border-white/10">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <span className="w-3 h-3 rounded-full bg-[#ef4444]/80 inline-block" />
            <span className="w-3 h-3 rounded-full bg-[#f59e0b]/80 inline-block" />
            <span className="w-3 h-3 rounded-full bg-[#10b981]/80 inline-block" />
          </div>
          <span className="ml-2 inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-semibold tracking-wider uppercase bg-[#f97316]/20 text-[#fb923c] border border-[#f97316]/30">
            <Sparkles className="w-3 h-3" />
            Coachify Commercial Control Plane · Spec 0041
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-2.5 text-xs">
          <span
            className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 font-medium"
            data-testid="signer-isolation-badge"
          >
            <ShieldCheck className="w-3.5 h-3.5" />
            Isolated Signer (k2026a · HMAC-SHA256)
          </span>
          <span
            className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-white/10 text-amber-200 border border-white/15 font-mono"
            data-testid="demo-totp-badge"
          >
            <Lock className="w-3 h-3 text-[#f97316]" />
            2FA TOTP: <strong>{overview?.demo_totp_code ?? "------"}</strong>
          </span>
          <button
            type="button"
            onClick={() => void loadOverview()}
            className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-white/10 hover:bg-white/20 text-white transition cursor-pointer"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
            Sync
          </button>
        </div>
      </div>

      {/* Hero Header Banner styled after landing pages/ */}
      <div className="px-6 py-5 bg-gradient-to-r from-[#f3ede2] via-[#faf7f2] to-[#f3ede2] border-b border-[#16110d]/10 flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-[#16110d]">
            Vendor Admin &amp; License Issuing Studio
          </h2>
          <p className="text-xs text-[#574c43] mt-0.5">
            Record bKash or Nagad payments, issue offline{" "}
            <code className="font-mono">.coachifylicense</code> files from{" "}
            <code className="font-mono">REQ-XXXX</code> codes, manage self-service PC
            transfers, and audit Ed25519 signatures.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="px-3.5 py-2 rounded-xl bg-white border border-[#16110d]/10 shadow-2xs text-right">
            <div className="text-[10px] uppercase tracking-wider font-semibold text-[#8a7c70]">
              Audit Hash Chain
            </div>
            <div className="text-xs font-bold text-emerald-700 flex items-center gap-1">
              <CheckCircle2 className="w-3.5 h-3.5" />
              {overview?.audit_chain_valid ? "SHA256 Verified" : "Checking"} (
              {overview?.admin_audit_log.length ?? 0} entries)
            </div>
          </div>
          <div className="px-3.5 py-2 rounded-xl bg-white border border-[#16110d]/10 shadow-2xs text-right">
            <div className="text-[10px] uppercase tracking-wider font-semibold text-[#8a7c70]">
              Signer Operations
            </div>
            <div className="text-xs font-bold text-[#16110d] font-mono">
              {overview?.signer_total_signatures ?? 0} signed blobs
            </div>
          </div>
        </div>
      </div>

      {/* Alerts */}
      {errorMessage && (
        <div className="mx-6 mt-4 p-3.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-900 text-xs font-medium flex items-center justify-between">
          <span>{errorMessage}</span>
          <button
            type="button"
            onClick={() => setErrorMessage(null)}
            className="text-xs underline ml-4 cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}
      {successBanner && (
        <div
          className="mx-6 mt-4 p-3.5 rounded-xl bg-emerald-500/15 border border-emerald-600/30 text-emerald-950 text-xs font-medium flex flex-wrap items-center justify-between gap-2"
          data-testid="vendor-success-banner"
        >
          <span>{successBanner}</span>
          <div className="flex items-center gap-2">
            {latestPairingCode && (
              <button
                type="button"
                onClick={() =>
                  void handleActivateLocalDesktopWithPairing(latestPairingCode)
                }
                className="px-3 py-1 rounded-full bg-[#f97316] text-white font-semibold hover:bg-[#ea580c] transition cursor-pointer"
                data-testid="instant-activate-pairing-btn"
              >
                Activate This PC Now ({latestPairingCode})
              </button>
            )}
            {latestSignedBlob && (
              <button
                type="button"
                onClick={() =>
                  handleDownloadBlob(latestSignedBlob.filename, latestSignedBlob.blob)
                }
                className="px-3 py-1 rounded-full bg-[#16110d] text-white font-semibold hover:bg-[#2c221b] transition inline-flex items-center gap-1 cursor-pointer"
                data-testid="download-latest-license-btn"
              >
                <Download className="w-3 h-3" />
                Download {latestSignedBlob.filename}
              </button>
            )}
          </div>
        </div>
      )}

      {/* 6-Tab Navigation Pill Bar */}
      <div className="px-6 pt-4 pb-2 flex flex-wrap gap-2 border-b border-[#16110d]/10 bg-[#f3ede2]/60">
        {[
          { id: "customers", label: "1. Customers & bKash Billing", icon: CreditCard },
          { id: "studio", label: "2. Offline License Studio", icon: QrCode },
          {
            id: "installations",
            label: "3. Installations & Transfers",
            icon: MonitorSmartphone,
          },
          { id: "revocations", label: "4. Revocations", icon: ShieldAlert },
          { id: "releases", label: "5. Releases & Channels", icon: Radio },
          { id: "audit", label: "6. Hash-Chained Audit & Signer", icon: FileText },
        ].map((tab) => {
          const Icon = tab.icon;
          const active = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              type="button"
              data-testid={`vendor-tab-${tab.id}`}
              onClick={() => setActiveTab(tab.id as VendorTab)}
              className={`inline-flex items-center gap-2 px-4 py-2 rounded-full text-xs font-semibold transition cursor-pointer ${
                active
                  ? "bg-[#f97316] text-white shadow-sm"
                  : "bg-white text-[#574c43] hover:bg-[#16110d]/5 border border-[#16110d]/10"
              }`}
            >
              <Icon className="w-3.5 h-3.5" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      <div className="p-6 space-y-6">
        {activeTab === "customers" && (
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            {/* Left 5 Cols: Record Payment & Issue License */}
            <div className="lg:col-span-5 space-y-5">
              <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold uppercase tracking-wider text-[#f97316]">
                    Manual Payment &amp; Auto-License
                  </span>
                  <span className="text-[11px] font-mono text-[#8a7c70]">
                    AC-1 / AC-6
                  </span>
                </div>
                <h3 className="text-base font-bold text-[#16110d]">
                  Record bKash / Nagad / Bank Payment
                </h3>
                <p className="text-xs text-[#574c43]">
                  Recording a payment extends{" "}
                  <code className="font-mono">paid_through</code>, calls the isolated{" "}
                  <code className="font-mono">license_signer</code>, and mints a 6-digit
                  online pairing code.
                </p>

                <div className="space-y-3 text-xs">
                  <div>
                    <label className="block font-semibold mb-1 text-[#16110d]">
                      Coaching Subscription
                    </label>
                    <select
                      value={paySubId}
                      onChange={(e) => {
                        const cust = overview?.customers.find(
                          (c) => c.subscription_id === e.target.value
                        );
                        setPaySubId(e.target.value);
                        if (cust) setPayCustomerId(cust.id);
                      }}
                      className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-medium"
                      data-testid="payment-subscription-select"
                    >
                      {overview?.customers.map((c) => (
                        <option key={c.subscription_id} value={c.subscription_id}>
                          {c.org_name} ({c.license_id} · {c.plan_code} · thru{" "}
                          {c.paid_through})
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="block font-semibold mb-1 text-[#16110d]">
                        Payment Channel
                      </label>
                      <select
                        value={payMethod}
                        onChange={(e) => setPayMethod(e.target.value)}
                        className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-medium"
                      >
                        <option value="bkash">bKash Merchant</option>
                        <option value="nagad">Nagad Merchant</option>
                        <option value="bank">Bank Transfer (BEFTN)</option>
                        <option value="cash">Field Cash Receipt</option>
                      </select>
                    </div>
                    <div>
                      <label className="block font-semibold mb-1 text-[#16110d]">
                        Amount (BDT)
                      </label>
                      <input
                        type="number"
                        value={payAmountBdt}
                        onChange={(e) => setPayAmountBdt(Number(e.target.value))}
                        className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-mono"
                      />
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="block font-semibold mb-1 text-[#16110d]">
                        Transaction Ref (TrxID)
                      </label>
                      <input
                        type="text"
                        value={payTxRef}
                        onChange={(e) => setPayTxRef(e.target.value)}
                        className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-mono"
                        data-testid="payment-tx-ref-input"
                      />
                    </div>
                    <div>
                      <label className="block font-semibold mb-1 text-[#16110d]">
                        Extension Period
                      </label>
                      <select
                        value={payPeriodMonths}
                        onChange={(e) => setPayPeriodMonths(Number(e.target.value))}
                        className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-medium"
                      >
                        <option value={1}>+1 Month</option>
                        <option value={6}>+6 Months</option>
                        <option value={12}>+12 Months (Annual)</option>
                      </select>
                    </div>
                  </div>

                  <button
                    type="button"
                    onClick={() => void handleRecordPayment()}
                    className="w-full py-2.5 px-4 rounded-full bg-[#f97316] hover:bg-[#ea580c] text-white font-semibold text-xs shadow-sm transition flex items-center justify-center gap-2 cursor-pointer"
                    data-testid="record-payment-submit-btn"
                  >
                    <CreditCard className="w-4 h-4" />
                    Record Payment &amp; Mint Signed License
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Add Coaching Customer Card */}
              <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
                <div className="flex items-center gap-2">
                  <Building2 className="w-4 h-4 text-[#f97316]" />
                  <h4 className="text-sm font-bold text-[#16110d]">
                    Onboard New Coaching Center
                  </h4>
                </div>
                <div className="grid grid-cols-2 gap-2.5 text-xs">
                  <input
                    type="text"
                    placeholder="Coaching Name (e.g. Apex Science)"
                    value={newOrgName}
                    onChange={(e) => setNewOrgName(e.target.value)}
                    className="rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                  <input
                    type="text"
                    placeholder="Owner Name"
                    value={newOwnerName}
                    onChange={(e) => setNewOwnerName(e.target.value)}
                    className="rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                  <input
                    type="text"
                    placeholder="Phone (+8801...)"
                    value={newPhone}
                    onChange={(e) => setNewPhone(e.target.value)}
                    className="rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 font-mono"
                  />
                  <select
                    value={newPlanCode}
                    onChange={(e) => setNewPlanCode(e.target.value)}
                    className="rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  >
                    <option value="local_pro">Coachify Local Pro (LAN)</option>
                    <option value="local">Coachify Local (Single PC)</option>
                  </select>
                </div>
                <button
                  type="button"
                  onClick={() => void handleCreateCustomer()}
                  className="w-full py-2 rounded-full bg-[#16110d] hover:bg-[#2c221b] text-white text-xs font-semibold transition flex items-center justify-center gap-1.5 cursor-pointer"
                >
                  <Plus className="w-3.5 h-3.5" />
                  Create Coaching Account &amp; Subscription
                </button>
              </div>
            </div>

            {/* Right 7 Cols: Active Subscriptions & Recent Payment Ledger */}
            <div className="lg:col-span-7 space-y-5">
              <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
                <h3 className="text-sm font-bold text-[#16110d]">
                  Active Coaching Subscriptions ({overview?.customers.length ?? 0})
                </h3>
                <div className="overflow-x-auto">
                  <table className="w-full text-xs border-collapse">
                    <thead>
                      <tr className="border-b border-[#16110d]/10 text-left text-[#8a7c70]">
                        <th className="py-2 pr-3">Coaching Center</th>
                        <th className="py-2 px-3">License / Pairing</th>
                        <th className="py-2 px-3">Plan</th>
                        <th className="py-2 px-3">Paid Through</th>
                        <th className="py-2 px-3">Serial</th>
                        <th className="py-2 pl-3 text-right">Online Action</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-[#16110d]/8">
                      {overview?.customers.map((c) => (
                        <tr key={c.id}>
                          <td className="py-2.5 pr-3 font-semibold text-[#16110d]">
                            {c.org_name}
                          </td>
                          <td className="py-2.5 px-3 font-mono text-[11px]">
                            {c.license_id}
                            {c.pairing_code ? ` (${c.pairing_code})` : ""}
                          </td>
                          <td className="py-2.5 px-3">
                            <span className="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase bg-[#f97316]/15 text-[#ea580c]">
                              {c.plan_code}
                            </span>
                          </td>
                          <td className="py-2.5 px-3 font-mono">{c.paid_through}</td>
                          <td className="py-2.5 px-3 font-mono">#{c.current_serial}</td>
                          <td className="py-2.5 pl-3 text-right">
                            <button
                              type="button"
                              onClick={() =>
                                void handleActivateLocalDesktopWithPairing(
                                  c.pairing_code ?? c.license_id
                                )
                              }
                              className="px-2.5 py-1 rounded-full bg-[#16110d] text-white text-[11px] font-semibold hover:bg-[#f97316] transition cursor-pointer"
                            >
                              Bind This PC
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
                <h3 className="text-sm font-bold text-[#16110d]">
                  Manual Payment Ledger ({overview?.payments.length ?? 0})
                </h3>
                <div className="overflow-x-auto">
                  <table className="w-full text-xs border-collapse">
                    <thead>
                      <tr className="border-b border-[#16110d]/10 text-left text-[#8a7c70]">
                        <th className="py-2 pr-3">Coaching</th>
                        <th className="py-2 px-3">Channel</th>
                        <th className="py-2 px-3">Invoice / TrxID</th>
                        <th className="py-2 px-3">Amount (BDT)</th>
                        <th className="py-2 pl-3">Extended</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-[#16110d]/8">
                      {overview?.payments.map((p) => (
                        <tr key={p.id}>
                          <td className="py-2 pr-3 font-medium">{p.org_name}</td>
                          <td className="py-2 px-3 uppercase font-mono text-[11px]">
                            {p.method}
                          </td>
                          <td className="py-2 px-3 font-mono text-[11px]">
                            {p.invoice_number} · {p.reference}
                          </td>
                          <td className="py-2 px-3 font-mono font-semibold">
                            ৳{Math.round(p.amount_poisha / 100).toLocaleString()}
                          </td>
                          <td className="py-2 pl-3 font-mono">+{p.period_months}m</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === "studio" && (
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div className="lg:col-span-5 p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold uppercase tracking-wider text-[#f97316]">
                  Air-Gapped &amp; Offline Issuing
                </span>
                <span className="text-[11px] font-mono text-[#8a7c70]">AC-3 / AC-4</span>
              </div>
              <h3 className="text-base font-bold text-[#16110d]">
                Issue Signed <code className="font-mono">.coachifylicense</code> from
                Request Code
              </h3>
              <p className="text-xs text-[#574c43]">
                Paste a <code className="font-mono">REQ-XXXX-XXXX-XXXX-XXXX</code> request
                code or <code className="font-mono">COACHIFY-REQ|v1|...</code> QR payload
                from any offline Coaching desktop.
              </p>

              <div className="space-y-3 text-xs">
                <div>
                  <label className="block font-semibold mb-1 text-[#16110d]">
                    Target License
                  </label>
                  <select
                    value={offlineLicenseId}
                    onChange={(e) => setOfflineLicenseId(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 text-xs font-medium"
                  >
                    {overview?.customers.map((c) => (
                      <option key={c.license_id} value={c.license_id}>
                        {c.org_name} ({c.license_id} · {c.plan_code})
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <div className="flex items-center justify-between mb-1">
                    <label className="font-semibold text-[#16110d]">
                      Request Code or QR Payload
                    </label>
                    <button
                      type="button"
                      onClick={() => void handleFillThisPcRequestCode()}
                      className="text-[11px] font-semibold text-[#f97316] hover:underline cursor-pointer"
                      data-testid="fill-this-pc-req-btn"
                    >
                      Use This PC&apos;s Request Code
                    </button>
                  </div>
                  <textarea
                    rows={3}
                    value={offlineRequestCode}
                    onChange={(e) => setOfflineRequestCode(e.target.value)}
                    placeholder="REQ-XXXX-XXXX-XXXX-XXXX or COACHIFY-REQ|v1|..."
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] p-3 font-mono text-xs"
                    data-testid="offline-req-code-input"
                  />
                </div>

                <button
                  type="button"
                  onClick={() => void handleIssueOfflineLicense()}
                  className="w-full py-2.5 px-4 rounded-full bg-[#f97316] hover:bg-[#ea580c] text-white font-semibold text-xs shadow-sm transition flex items-center justify-center gap-2 cursor-pointer"
                  data-testid="issue-offline-license-btn"
                >
                  <KeyRound className="w-4 h-4" />
                  Validate Checksum &amp; Sign .coachifylicense
                </button>
              </div>
            </div>

            <div className="lg:col-span-7 p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
              <h3 className="text-sm font-bold text-[#16110d]">
                Active Customer Licenses &amp; Downloadable Artifacts
              </h3>
              <div className="space-y-2.5 max-h-[420px] overflow-y-auto pr-1">
                {overview?.customers.map((cust) => (
                  <div
                    key={cust.id}
                    className="p-3.5 rounded-xl bg-[#faf7f2] border border-[#16110d]/10 flex flex-wrap items-center justify-between gap-3 text-xs"
                  >
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-bold text-[#16110d]">{cust.org_name}</span>
                        <span className="font-mono text-[11px] px-2 py-0.5 rounded bg-white border border-[#16110d]/10">
                          {cust.license_id} · Serial #{cust.current_serial}
                        </span>
                      </div>
                      <div className="text-[11px] text-[#574c43] font-mono">
                        Bound PC: {cust.bound_installation_id ?? "Unbound"} · Paid
                        Through: {cust.paid_through}
                        {cust.pairing_code ? ` · Pairing: ${cust.pairing_code}` : ""}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {cust.pairing_code && (
                        <button
                          type="button"
                          onClick={() => void handleCopy(cust.pairing_code!, cust.id)}
                          className="px-2.5 py-1 rounded-full bg-white border border-[#16110d]/15 text-[11px] font-mono inline-flex items-center gap-1 cursor-pointer"
                        >
                          {copiedId === cust.id ? (
                            <Check className="w-3 h-3 text-emerald-600" />
                          ) : (
                            <Copy className="w-3 h-3" />
                          )}
                          {cust.pairing_code}
                        </button>
                      )}
                      {cust.last_issued_blob && (
                        <button
                          type="button"
                          onClick={() =>
                            handleDownloadBlob(
                              `${cust.license_id}-serial-${cust.current_serial}.coachifylicense`,
                              cust.last_issued_blob!
                            )
                          }
                          className="px-3 py-1 rounded-full bg-[#16110d] text-white text-[11px] font-semibold inline-flex items-center gap-1 hover:bg-[#f97316] transition cursor-pointer"
                        >
                          <Download className="w-3 h-3" />
                          .coachifylicense
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === "installations" && (
          <div className="space-y-5">
            <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div>
                  <h3 className="text-sm font-bold text-[#16110d]">
                    Bound Hardware Installations &amp; 12-Month Transfer Cap (Max 3 / 12m)
                  </h3>
                  <p className="text-xs text-[#574c43]">
                    Self-service PC transfers automatically deactivate the previous
                    installation and increment{" "}
                    <code className="font-mono">transfer_count_12m</code>.
                  </p>
                </div>
              </div>

              <div className="overflow-x-auto">
                <table className="w-full text-xs border-collapse">
                  <thead>
                    <tr className="border-b border-[#16110d]/10 text-left text-[#8a7c70]">
                      <th className="py-2 pr-3">Coaching</th>
                      <th className="py-2 px-3">License ID</th>
                      <th className="py-2 px-3">Hardware Installation ID</th>
                      <th className="py-2 px-3">Device Label</th>
                      <th className="py-2 px-3">Status</th>
                      <th className="py-2 px-3">Transfers Used</th>
                      <th className="py-2 pl-3 text-right">Vendor Override</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[#16110d]/8">
                    {overview?.installations.map((inst) => (
                      <tr key={inst.id}>
                        <td className="py-2.5 pr-3 font-semibold">{inst.org_name}</td>
                        <td className="py-2.5 px-3 font-mono">{inst.license_id}</td>
                        <td className="py-2.5 px-3 font-mono text-[11px]">
                          {inst.installation_id}
                        </td>
                        <td className="py-2.5 px-3">{inst.label}</td>
                        <td className="py-2.5 px-3">
                          <span
                            className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase ${
                              inst.status === "active"
                                ? "bg-emerald-500/15 text-emerald-800"
                                : "bg-zinc-500/15 text-zinc-700"
                            }`}
                          >
                            {inst.status}
                          </span>
                        </td>
                        <td className="py-2.5 px-3 font-mono">
                          {inst.transfer_count_12m} / 3
                        </td>
                        <td className="py-2.5 pl-3 text-right">
                          <button
                            type="button"
                            onClick={() => void handleResetTransferCount(inst.license_id)}
                            className="px-2.5 py-1 rounded-full bg-[#faf7f2] border border-[#16110d]/15 hover:bg-[#f97316] hover:text-white text-[11px] font-semibold transition cursor-pointer"
                          >
                            Reset Cap (2FA)
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}

        {activeTab === "revocations" && (
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div className="lg:col-span-5 p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold uppercase tracking-wider text-red-600">
                  Step-Up 2FA Revocation
                </span>
                <span className="text-[11px] font-mono text-[#8a7c70]">AC-9</span>
              </div>
              <h3 className="text-base font-bold text-[#16110d]">
                Sign Cryptographic Revocation Certificate
              </h3>
              <p className="text-xs text-[#574c43]">
                Requires valid TOTP verification. Calls{" "}
                <code className="font-mono">license_signer</code> to sign an Ed25519
                revocation certificate that locks target desktops on their next refresh.
              </p>

              <div className="space-y-3 text-xs">
                <div>
                  <label className="block font-semibold mb-1">License ID</label>
                  <input
                    type="text"
                    value={revokeLicenseId}
                    onChange={(e) => setRevokeLicenseId(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 font-mono"
                  />
                </div>
                <div>
                  <label className="block font-semibold mb-1">Revocation Reason</label>
                  <input
                    type="text"
                    value={revokeReason}
                    onChange={(e) => setRevokeReason(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                </div>
                <div>
                  <label className="block font-semibold mb-1">
                    Step-Up 2FA TOTP Code (Current: {overview?.demo_totp_code})
                  </label>
                  <input
                    type="text"
                    value={stepUpTotp}
                    onChange={(e) => setStepUpTotp(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 font-mono"
                  />
                </div>
                <button
                  type="button"
                  onClick={() => void handleRevokeLicense()}
                  className="w-full py-2.5 px-4 rounded-full bg-red-600 hover:bg-red-700 text-white font-semibold text-xs transition cursor-pointer"
                  data-testid="revoke-license-submit-btn"
                >
                  Sign &amp; Publish Revocation Certificate
                </button>
              </div>
            </div>

            <div className="lg:col-span-7 p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
              <h3 className="text-sm font-bold text-[#16110d]">
                Signed Revocation Registry ({overview?.revocations.length ?? 0})
              </h3>
              <div className="space-y-2.5">
                {overview?.revocations.map((rev) => (
                  <div
                    key={rev.id}
                    className="p-3.5 rounded-xl bg-red-500/5 border border-red-500/20 text-xs space-y-1"
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-bold text-red-900 font-mono">
                        {rev.license_id} ({rev.org_name})
                      </span>
                      <span className="font-mono text-[11px] text-red-700">
                        Key: {rev.key_id} · {rev.revoked_at}
                      </span>
                    </div>
                    <p className="text-[#574c43]">Reason: {rev.reason}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === "releases" && (
          <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-4">
            <div>
              <h3 className="text-sm font-bold text-[#16110d]">
                Update Channels &amp; Signed Release Manifests (
                <code className="font-mono">GET /v1/updates/&#123;channel&#125;</code>)
              </h3>
              <p className="text-xs text-[#574c43]">
                Control <code className="font-mono">stable</code> and{" "}
                <code className="font-mono">beta</code> rollout status. Blocking a release
                requires Step-Up 2FA TOTP.
              </p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {overview?.releases.map((rel) => (
                <div
                  key={rel.id}
                  className="p-4 rounded-2xl bg-[#faf7f2] border border-[#16110d]/10 space-y-2.5 text-xs"
                >
                  <div className="flex items-center justify-between">
                    <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase bg-[#16110d] text-white">
                      {rel.channel} channel
                    </span>
                    <span className="font-mono font-bold text-sm">v{rel.version}</span>
                  </div>
                  <div className="text-[11px] font-mono text-[#574c43] truncate">
                    SHA256: {rel.sha256}
                  </div>
                  <p className="text-xs text-[#16110d]">{rel.notes}</p>
                  <div className="flex items-center justify-between pt-2 border-t border-[#16110d]/10">
                    <span className="font-semibold uppercase text-[11px]">
                      Status: <strong>{rel.status}</strong>
                    </span>
                    <div className="flex items-center gap-1.5">
                      {["live", "held", "blocked"].map((st) => (
                        <button
                          key={st}
                          type="button"
                          onClick={() =>
                            void handleUpdateReleaseStatus(rel.version, rel.channel, st)
                          }
                          className={`px-2.5 py-1 rounded-full text-[10px] font-semibold uppercase cursor-pointer ${
                            rel.status === st
                              ? "bg-[#f97316] text-white"
                              : "bg-white border border-[#16110d]/15 text-[#574c43]"
                          }`}
                        >
                          {st}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Zero-Rebuild Channel Promotion (Spec 0042b AC-6) */}
            <div className="mt-6 p-5 rounded-2xl bg-[#faf7f2] border border-[#16110d]/10 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <span className="text-xs font-bold uppercase tracking-wider text-orange-600">
                    Zero-Rebuild Channel Promotion · Spec 0042b AC-6
                  </span>
                  <h4 className="text-sm font-bold text-[#16110d]">
                    Promote Release Artifact Across Channels
                  </h4>
                  <p className="text-xs text-[#574c43]">
                    Promote an immutable release from{" "}
                    <code className="font-mono">internal</code> to{" "}
                    <code className="font-mono">beta</code> or{" "}
                    <code className="font-mono">stable</code> without rebuilding binaries.
                    Promoting to stable requires Step-Up 2FA TOTP.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-4 gap-3 text-xs">
                <div>
                  <label className="block font-semibold mb-1">Release Version</label>
                  <input
                    type="text"
                    value={promoteVersion}
                    onChange={(e) => setPromoteVersion(e.target.value)}
                    placeholder="1.0.0"
                    className="w-full rounded-xl border border-[#16110d]/15 bg-white px-3 py-2 font-mono"
                  />
                </div>
                <div>
                  <label className="block font-semibold mb-1">Source Channel</label>
                  <select
                    value={promoteSourceChannel}
                    onChange={(e) => setPromoteSourceChannel(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-white px-3 py-2"
                  >
                    <option value="internal">internal (Pre-Release Approval)</option>
                    <option value="beta">beta (Early Access)</option>
                  </select>
                </div>
                <div>
                  <label className="block font-semibold mb-1">Target Channel</label>
                  <select
                    value={promoteTargetChannel}
                    onChange={(e) =>
                      setPromoteTargetChannel(e.target.value as "beta" | "stable")
                    }
                    className="w-full rounded-xl border border-[#16110d]/15 bg-white px-3 py-2 font-bold"
                  >
                    <option value="beta">beta (Early Access)</option>
                    <option value="stable">
                      stable (General Availability - 2FA Req)
                    </option>
                  </select>
                </div>
                <div>
                  <label className="block font-semibold mb-1">
                    Step-Up 2FA TOTP (Current: {overview?.demo_totp_code})
                  </label>
                  <input
                    type="text"
                    value={stepUpTotp}
                    onChange={(e) => setStepUpTotp(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-white px-3 py-2 font-mono"
                  />
                </div>
              </div>

              <div className="text-xs">
                <label className="block font-semibold mb-1">
                  Promotion Reason &amp; Audit Rationale
                </label>
                <input
                  type="text"
                  value={promoteReason}
                  onChange={(e) => setPromoteReason(e.target.value)}
                  placeholder="Passed full verification matrix and verified on clean Windows 11 host"
                  className="w-full rounded-xl border border-[#16110d]/15 bg-white px-3 py-2"
                />
              </div>

              <div className="flex justify-end">
                <button
                  type="button"
                  onClick={() => void handlePromoteRelease()}
                  className="px-5 py-2.5 rounded-full bg-orange-600 hover:bg-orange-700 text-white font-semibold text-xs transition cursor-pointer shadow-sm"
                  data-testid="promote-release-submit-btn"
                >
                  Promote Release (Zero Rebuild) →
                </button>
              </div>
            </div>

            {/* Release Channel Heads (Spec 0042b AC-6) */}
            {overview?.channel_heads && overview.channel_heads.length > 0 && (
              <div className="p-4 rounded-2xl bg-white border border-[#16110d]/10 space-y-3 text-xs">
                <h4 className="font-bold text-[#16110d]">
                  Active Channel Heads (
                  <code className="font-mono">release_channel_heads</code>)
                </h4>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  {overview.channel_heads.map((head) => (
                    <div
                      key={head.channel}
                      className="p-3 rounded-xl border border-[#16110d]/10 bg-[#faf7f2] space-y-1.5"
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-bold uppercase text-[11px] px-2 py-0.5 rounded bg-[#16110d] text-white">
                          {head.channel}
                        </span>
                        <span className="font-mono font-bold text-orange-600">
                          v{head.version}
                        </span>
                      </div>
                      <p className="text-[11px] font-mono text-[#574c43] truncate">
                        SHA256: {head.sha256}
                      </p>
                      <p className="text-[10px] text-[#8a7c70]">
                        Promoted: {head.promoted_at} by {head.promoted_by}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Key Recovery Drill Studio (Spec 0042b AC-8) */}
            <div className="mt-6 p-5 rounded-2xl bg-white border border-[#16110d]/10 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <span className="text-xs font-bold uppercase tracking-wider text-emerald-700">
                    Cryptographic Resilience · Spec 0042b AC-8
                  </span>
                  <h4 className="text-sm font-bold text-[#16110d]">
                    Key Custody &amp; Recovery Drill Studio
                  </h4>
                  <p className="text-xs text-[#574c43]">
                    Perform and record disaster recovery drills under dual-custody.
                    Records are stored in append-only SQLite tables protected by database
                    triggers.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
                <div>
                  <label className="block font-semibold mb-1">Key Type</label>
                  <select
                    value={drillKeyType}
                    onChange={(e) => {
                      const nextType = e.target.value as
                        | "windows_authenticode"
                        | "tauri_updater_ed25519"
                        | "license_ed25519";
                      setDrillKeyType(nextType);
                      setDrillKeyId(
                        nextType === "windows_authenticode"
                          ? "auth2026a"
                          : nextType === "tauri_updater_ed25519"
                            ? "u2026a"
                            : "k2026a"
                      );
                    }}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  >
                    <option value="windows_authenticode">
                      Windows Authenticode (SignTool)
                    </option>
                    <option value="tauri_updater_ed25519">
                      Tauri Updater Ed25519 (u2026a)
                    </option>
                    <option value="license_ed25519">
                      License Signer Ed25519 (k2026a)
                    </option>
                  </select>
                </div>
                <div>
                  <label className="block font-semibold mb-1">Primary Custodian</label>
                  <input
                    type="text"
                    value={drillPrimaryCustodian}
                    onChange={(e) => setDrillPrimaryCustodian(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                </div>
                <div>
                  <label className="block font-semibold mb-1">Secondary Custodian</label>
                  <input
                    type="text"
                    value={drillSecondaryCustodian}
                    onChange={(e) => setDrillSecondaryCustodian(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                <div>
                  <label className="block font-semibold mb-1">Test Artifact SHA256</label>
                  <input
                    type="text"
                    value={drillTestSha}
                    onChange={(e) => setDrillTestSha(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2 font-mono text-[11px]"
                  />
                </div>
                <div>
                  <label className="block font-semibold mb-1">
                    Drill Notes &amp; Findings
                  </label>
                  <input
                    type="text"
                    value={drillNotes}
                    onChange={(e) => setDrillNotes(e.target.value)}
                    className="w-full rounded-xl border border-[#16110d]/15 bg-[#faf7f2] px-3 py-2"
                  />
                </div>
              </div>

              <div className="flex justify-end">
                <button
                  type="button"
                  onClick={() => void handleRecordKeyRecoveryDrill()}
                  className="px-5 py-2.5 rounded-full bg-emerald-700 hover:bg-emerald-800 text-white font-semibold text-xs transition cursor-pointer shadow-sm"
                  data-testid="record-key-drill-btn"
                >
                  Record &amp; Verify Key Recovery Drill →
                </button>
              </div>

              {/* Append-Only Drill Log */}
              {overview?.key_recovery_drills &&
                overview.key_recovery_drills.length > 0 && (
                  <div className="mt-4 border-t border-[#16110d]/10 pt-4 space-y-2">
                    <h5 className="font-bold text-[#16110d] text-xs">
                      Recorded Drills (Append-Only Log)
                    </h5>
                    <div className="space-y-2">
                      {overview.key_recovery_drills.map((drill) => (
                        <div
                          key={drill.id}
                          className="p-3 rounded-xl border border-emerald-500/20 bg-emerald-500/5 text-xs flex flex-wrap items-center justify-between gap-2"
                        >
                          <div>
                            <span className="font-bold text-emerald-950 font-mono">
                              {drill.key_type} ({drill.key_id})
                            </span>
                            <p className="text-[11px] text-[#574c43]">
                              Custodians: {drill.custodian_primary} &amp;{" "}
                              {drill.custodian_secondary} · {drill.drilled_at}
                            </p>
                            <p className="text-[10px] text-stone-500">{drill.notes}</p>
                          </div>
                          <span className="rounded-full bg-emerald-200 px-2 py-0.5 text-[10px] font-bold text-emerald-900">
                            {drill.status}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
            </div>
          </div>
        )}

        {activeTab === "audit" && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-bold text-[#16110d]">
                  Admin Audit Log (SHA256 Hash-Chained · AC-5)
                </h3>
                <span className="text-[11px] font-mono text-emerald-700 font-semibold">
                  {overview?.audit_chain_valid ? "CHAIN INTACT" : "CHAIN ERROR"}
                </span>
              </div>
              <div className="space-y-2 max-h-[380px] overflow-y-auto pr-1">
                {overview?.admin_audit_log.map((row) => (
                  <div
                    key={row.seq}
                    className="p-3 rounded-xl bg-[#faf7f2] border border-[#16110d]/10 text-xs space-y-1"
                  >
                    <div className="flex items-center justify-between font-semibold">
                      <span>
                        #{row.seq} · {row.action}
                      </span>
                      <span className="font-mono text-[10px] text-[#8a7c70]">
                        {row.entity_type}:{row.entity_id}
                      </span>
                    </div>
                    <div className="text-[10px] font-mono text-[#574c43] truncate">
                      prev: {row.prev_hash.slice(0, 16)}… → hash: {row.hash.slice(0, 16)}…
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="p-5 rounded-2xl bg-white border border-[#16110d]/10 shadow-xs space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-bold text-[#16110d]">
                  Isolated Signer Audit Log (
                  <code className="font-mono">crates/license_signer</code> · AC-2)
                </h3>
                <span className="text-[11px] font-mono text-[#f97316] font-semibold">
                  APPEND-ONLY
                </span>
              </div>
              <div className="space-y-2 max-h-[380px] overflow-y-auto pr-1">
                {overview?.signer_audit_log.map((s) => (
                  <div
                    key={s.seq}
                    className="p-3 rounded-xl bg-[#faf7f2] border border-[#16110d]/10 text-xs space-y-1"
                  >
                    <div className="flex items-center justify-between font-semibold">
                      <span>
                        #{s.seq} · {s.operation} ({s.key_id})
                      </span>
                      <span className="font-mono text-[10px] text-emerald-700">
                        {s.license_id} · Serial #{s.serial ?? 0}
                      </span>
                    </div>
                    <div className="text-[10px] font-mono text-[#574c43] truncate">
                      sha256: {s.payload_sha256.slice(0, 20)}… · sig: {s.signature_prefix}
                      …
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
