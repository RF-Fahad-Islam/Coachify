import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { TodayCockpitView } from "./today-cockpit-view";
import { CoachingDayView } from "./coaching-day-view";
import { BatchHealthView } from "./batch-health-view";
import { BatchProfitabilityView } from "./batch-profitability-view";
import { SmartDueQueueView } from "./smart-due-queue-view";
import { IntelligenceHub } from "./intelligence-hub";
import { Student360Timeline } from "../students/student-360-timeline";

describe("Owner Intelligence Components (Slice 7: Features 34-39, Spec 0033)", () => {
  it("renders TodayCockpitView header, controls, and KPI labels", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <TodayCockpitView />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Daily Operations Cockpit");
    expect(html).toContain(
      "Real-time coaching center operational pulse: attendance vigilance"
    );
    expect(html).toContain("Live Today");
  });

  it("renders CoachingDayView header and all 7 category filters", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <CoachingDayView />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Coaching Day Timeline");
    expect(html).toContain(
      "Immutable single-day activity ledger: counter collections, admissions"
    );
    expect(html).toContain("All Activity");
    expect(html).toContain("Finance &amp; Cash");
    expect(html).toContain("Admissions");
    expect(html).toContain("Attendance");
    expect(html).toContain("Exams &amp; Marks");
    expect(html).toContain("Comms &amp; SMS");
    expect(html).toContain("Administrative");
  });

  it("renders BatchHealthView header, program selector, and risk KPI cards", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <BatchHealthView />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Batch Health &amp; Operational Risk Matrix");
    expect(html).toContain("Monitored Batches");
    expect(html).toContain("Healthy Batches");
    expect(html).toContain("Attention Required");
    expect(html).toContain("Critical Cohorts");
    expect(html).toContain("All Programs");
  });

  it("renders BatchProfitabilityView header and portfolio unit economics", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <BatchProfitabilityView />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Batch Profitability &amp; Unit Economics");
    expect(html).toContain("Projected Tuition");
    expect(html).toContain("Net Collections");
    expect(html).toContain("Cohort Operating Costs");
    expect(html).toContain("Portfolio Net Margin");
    expect(html).toContain("All Batches");
  });

  it("renders SmartDueQueueView header, filters, and priority cards", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <SmartDueQueueView />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Smart Due Recovery Queue");
    expect(html).toContain("Queued Defaulters");
    expect(html).toContain("Total Recoverable Dues");
    expect(html).toContain("High Urgent Priority");
    expect(html).toContain("All Overdue Days");
    expect(html).toContain("&gt; 30 Days Overdue");
  });

  it("renders IntelligenceHub unifying container with all 5 module tabs", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <IntelligenceHub />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Owner Intelligence &amp; Operations");
    expect(html).toContain("Slice 7");
    expect(html).toContain("Today Cockpit");
    expect(html).toContain("Coaching Day");
    expect(html).toContain("Batch Health");
    expect(html).toContain("Batch Profitability");
    expect(html).toContain("Smart Due Queue");
  });

  it("renders Student360Timeline journey categories and Add Note trigger", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <Student360Timeline studentId="stu-test-123" />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Full Journey");
    expect(html).toContain("Payments &amp; Dues");
    expect(html).toContain("Attendance");
    expect(html).toContain("Exams &amp; Results");
    expect(html).toContain("Enrollments");
    expect(html).toContain("SMS &amp; Outbox");
    expect(html).toContain("Teacher Notes");
    expect(html).toContain("Add Note");
  });
});
