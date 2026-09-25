import fs from "node:fs";
import path from "node:path";
import { describe, it, expect, vi } from "vitest";
import { renderToString } from "react-dom/server";
import {
  getCustomerDocs,
  filterCustomerDocs,
  buildPrintableGuideHtml,
  printCustomerGuideAsPdf,
} from "@/lib/docs";
import { HelpDialog } from "./help-dialog";
import { TopBar } from "@/components/layout/top-bar";

const REPO_ROOT = path.resolve(__dirname, "../../../../");

describe("Feature 60: Customer Documentation, Support Runbook & Offline Help Modal", () => {
  it("loads all 8 customer guides in order via Vite raw import with zero network calls (AC-8, AC-9)", () => {
    const docs = getCustomerDocs();
    expect(docs).toHaveLength(8);
    expect(docs.map((d) => d.filename)).toEqual([
      "01-quick-start.md",
      "02-backup-and-restore.md",
      "03-printer-setup.md",
      "04-sms-setup.md",
      "05-excel-student-import.md",
      "06-troubleshooting.md",
      "07-privacy-and-ownership.md",
      "08-license-guide.md",
    ]);

    for (const doc of docs) {
      expect(doc.content.length).toBeGreaterThan(500);
      expect(doc.headings.length).toBeGreaterThanOrEqual(3);
      expect(doc.titleBn.length).toBeGreaterThan(0);
    }
  });

  it("excludes internal support-runbook.md from customer-facing topics while enforcing runbook security invariants (AC-10)", () => {
    const docs = getCustomerDocs();
    expect(docs.some((d) => d.filename === "support-runbook.md")).toBe(false);

    const runbookPath = path.join(REPO_ROOT, "docs", "customer", "support-runbook.md");
    expect(fs.existsSync(runbookPath)).toBe(true);

    const runbookContent = fs.readFileSync(runbookPath, "utf-8");
    expect(runbookContent).toContain("Mandatory Identity Verification Gate");
    expect(runbookContent).toContain("acknowledge_clock_jump");
    expect(runbookContent).toContain("DPAPI");
    expect(runbookContent).toContain("coachify.db-wal");
    // Must never contain hardcoded private keys or master secrets
    expect(runbookContent).not.toMatch(/BEGIN PRIVATE KEY/i);
    expect(runbookContent).not.toMatch(/MASTER_KEY\s*=/i);
  });

  it("filters customer guides accurately across English, Bengali, and body keywords (AC-8)", () => {
    const docs = getCustomerDocs();

    const smartScreenMatches = filterCustomerDocs(docs, "SmartScreen");
    expect(smartScreenMatches.some((d) => d.id === "06-troubleshooting")).toBe(true);

    const banglaPrinterMatches = filterCustomerDocs(docs, "প্রিন্টার");
    expect(banglaPrinterMatches.some((d) => d.id === "03-printer-setup")).toBe(true);

    const noMatches = filterCustomerDocs(docs, "nonexistent_topic_xyz_999999");
    expect(noMatches).toHaveLength(0);
  });

  it("generates formatted A4 printable HTML and triggers WebView2 print engine (AC-9)", async () => {
    const printSpy = vi.fn().mockResolvedValue({ success: true });

    const docs = getCustomerDocs();
    for (const topic of docs) {
      const html = buildPrintableGuideHtml(topic);
      expect(html).toContain("<!DOCTYPE html>");
      expect(html).toContain("@page { size: A4 portrait;");
      expect(html).toContain(topic.shortTitle.replace(/&/g, "&amp;"));
    }

    const res = await printCustomerGuideAsPdf(docs[0], printSpy);
    expect(res.success).toBe(true);
    expect(printSpy).toHaveBeenCalledTimes(1);
  });

  it("renders HelpDialog with all 8 topics, active topic article, search filtering, and PDF print button (AC-8, AC-9)", () => {
    const defaultHtml = renderToString(
      <HelpDialog open={true} onOpenChange={() => {}} />
    );

    expect(defaultHtml).toContain("Coachify Help &amp; Documentation");
    expect(defaultHtml).toContain("100% Offline Ready");
    expect(defaultHtml).toContain("Quick Start Guide");
    expect(defaultHtml).toContain("Backup and Restore");
    expect(defaultHtml).toContain("Printer Setup");
    expect(defaultHtml).toContain("SMS Setup");
    expect(defaultHtml).toContain("Excel Import");
    expect(defaultHtml).toContain("Troubleshooting");
    expect(defaultHtml).toContain("Privacy &amp; Ownership");
    expect(defaultHtml).toContain("License &amp; Activation");
    expect(defaultHtml).toContain("Print Guide as PDF");

    // Verify rendering a specific topic & search filter
    const troubleshootingHtml = renderToString(
      <HelpDialog
        open={true}
        onOpenChange={() => {}}
        initialTopicId="06-troubleshooting"
        initialSearchQuery="CR2032"
      />
    );
    expect(troubleshootingHtml).toContain(
      "Bypassing Windows SmartScreen During Installation"
    );
    expect(troubleshootingHtml).toContain("Topics (1)");
  });

  it("renders the Help trigger button in TopBar (AC-8)", () => {
    const html = renderToString(
      <TopBar
        onOpenThemeCustomizer={() => {}}
        onSearchClick={() => {}}
        onOpenHelp={() => {}}
      />
    );

    expect(html).toContain('data-testid="open-help-btn"');
    expect(html).toContain("Open Help &amp; Documentation");
  });
});

describe("Feature 61: 1.0 Readiness Qualification Report & Pilot Matrix Verification", () => {
  it("validates pilot-matrix.json meets all 4-week, 3-site, zero-data-loss qualification invariants (AC-12)", () => {
    const matrixPath = path.join(REPO_ROOT, "docs", "release", "pilot-matrix.json");
    expect(fs.existsSync(matrixPath)).toBe(true);

    const raw = JSON.parse(fs.readFileSync(matrixPath, "utf-8")) as {
      pilot_program: {
        target_duration_weeks: number;
        required_sites_count: number;
        qualification_status: string;
        total_data_loss_incidents: number;
        sites: Array<{
          id: string;
          name: string;
          student_count: number;
          duration_weeks: number;
          monthly_fee_cycle_completed: boolean;
          exam_cycle_completed: boolean;
          restore_drill_completed: boolean;
          data_loss_incidents: number;
          data_verification_hash: string;
          owner_sign_off: string;
          restore_drill_log: {
            pre_restore_student_count: number;
            post_restore_student_count: number;
            pre_restore_ledger_balance_paisa: number;
            post_restore_ledger_balance_paisa: number;
            sqlite_integrity_check: string;
            audit_chain_verified: boolean;
          };
        }>;
      };
    };

    const program = raw.pilot_program;
    expect(program.target_duration_weeks).toBeGreaterThanOrEqual(4);
    expect(program.required_sites_count).toBeGreaterThanOrEqual(3);
    expect(program.sites.length).toBeGreaterThanOrEqual(3);
    expect(program.total_data_loss_incidents).toBe(0);

    for (const site of program.sites) {
      expect(site.duration_weeks).toBeGreaterThanOrEqual(4);
      expect(site.monthly_fee_cycle_completed).toBe(true);
      expect(site.exam_cycle_completed).toBe(true);
      expect(site.restore_drill_completed).toBe(true);
      expect(site.data_loss_incidents).toBe(0);
      expect(site.owner_sign_off).toBe("Verified");
      expect(site.data_verification_hash).toMatch(/^[a-f0-9]{64}$/);

      // Restore drill parity check
      expect(site.restore_drill_log.post_restore_student_count).toBe(
        site.restore_drill_log.pre_restore_student_count
      );
      expect(site.restore_drill_log.post_restore_ledger_balance_paisa).toBe(
        site.restore_drill_log.pre_restore_ledger_balance_paisa
      );
      expect(site.restore_drill_log.sqlite_integrity_check).toBe("ok");
      expect(site.restore_drill_log.audit_chain_verified).toBe(true);
    }
  });

  it("validates 1.0-readiness-report.md covers all 14 release criteria with PASS status (AC-11)", () => {
    const reportPath = path.join(REPO_ROOT, "docs", "release", "1.0-readiness-report.md");
    expect(fs.existsSync(reportPath)).toBe(true);

    const report = fs.readFileSync(reportPath, "utf-8");
    expect(report).toContain("APPROVED FOR 1.0 GA");
    expect(report).toContain("14 / 14 Passed");
    expect(report).toContain("deferred to version `1.1`");

    const requiredCriteria = [
      "Scope completion",
      "Automated test suite",
      "Financial property tests",
      "Crash safety suite",
      "Backup & restore drill",
      "Installation & upgrade",
      "Printer compatibility",
      "Multi-week pilot trials",
      "Field restore drills",
      "Defect status",
      "License issuance",
      "Code signing",
      "Documentation suite",
      "Security audit",
    ];

    for (const criterion of requiredCriteria) {
      expect(report).toContain(criterion);
    }
  });
});
