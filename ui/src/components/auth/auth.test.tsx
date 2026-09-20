import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { AuthProvider } from "@/contexts/auth-context";
import { ThemeProvider } from "@/theme/theme-provider";
import { IdleLockDialog } from "./idle-lock-dialog";
import { BootstrapOwnerDialog } from "./bootstrap-owner-dialog";
import { LoginDialog } from "./login-dialog";
import { OnboardingCarousel } from "./onboarding-carousel";
import { StartupAuthPage } from "./startup-auth-page";

describe("Auth Dialog & Full-Page Components", () => {
  it("renders dialogs without crashing inside AuthProvider", () => {
    const html = renderToString(
      <AuthProvider>
        <div>
          <IdleLockDialog />
          <BootstrapOwnerDialog />
          <LoginDialog />
        </div>
      </AuthProvider>
    );
    expect(html).toBeDefined();
  });

  it("renders OnboardingCarousel with slides and feature highlights", () => {
    const html = renderToString(<OnboardingCarousel />);
    expect(html).toContain("Coachify Desktop");
    expect(html).toContain("v0.1.0 Offline Core");
    expect(html).toContain("Instant Counter Speed");
  });

  it("renders StartupAuthPage cleanly with onboarding and auth form", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <StartupAuthPage />
        </AuthProvider>
      </ThemeProvider>
    );
    expect(html).toContain("Coachify Desktop");
    expect(html).toContain("Sign in to Coachify");
    expect(html).toContain("Sign In to Counter");
  });
});
