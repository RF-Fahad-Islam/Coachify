import { useState, type FormEvent } from "react";
import { useAuth } from "@/contexts/auth-context";
import { OnboardingCarousel } from "./onboarding-carousel";
import { ThemeCustomizerDialog } from "@/components/theme/theme-customizer-dialog";
import { SetupWizard } from "@/components/setup/setup-wizard";
import { EmergencyRecoveryDialog } from "@/components/setup/emergency-recovery-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import {
  LogIn,
  ArrowRight,
  AlertCircle,
  Palette,
  Sparkles,
  Lock,
  User,
  Building2,
  CheckCircle2,
  KeyRound,
} from "lucide-react";

export function StartupAuthPage() {
  const { needsSetup, onSetupComplete, login, error, clearError, refreshStatus } =
    useAuth();
  const [customizerOpen, setCustomizerOpen] = useState(false);
  const [emergencyOpen, setEmergencyOpen] = useState(false);
  const [recoverySuccessMsg, setRecoverySuccessMsg] = useState<string | null>(null);

  // Login form state
  const [loginUsername, setLoginUsername] = useState("");
  const [loginPassword, setLoginPassword] = useState("");

  const [submitting, setSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  // Login submit handler
  const handleLoginSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (!loginUsername.trim() || !loginPassword) {
      setLocalError("Please enter your username and password.");
      return;
    }

    setSubmitting(true);
    await login(loginUsername.trim(), loginPassword);
    setSubmitting(false);
  };

  if (needsSetup) {
    return <SetupWizard onComplete={(user, token) => onSetupComplete(user, token)} />;
  }

  const activeError = localError || error;

  return (
    <div className="min-h-screen w-full flex flex-col lg:flex-row bg-background text-foreground antialiased select-none">
      {/* Left Column: Onboarding & Brand Showcase (Fatty Bento Visuals) */}
      <div className="w-full lg:w-3/5 bg-gradient-to-br from-primary via-primary to-primary-hover text-primary-foreground border-b lg:border-b-0 lg:border-r border-primary-hover/30 flex flex-col justify-between relative overflow-hidden">
        <OnboardingCarousel />
      </div>

      {/* Right Column: Authentication Card Container */}
      <div className="w-full lg:w-2/5 flex flex-col justify-between p-6 sm:p-10 lg:p-12 relative bg-card/30">
        {/* Top bar controls on right side */}
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-xs font-semibold text-muted-foreground">
              Local Counter Active
            </span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCustomizerOpen(true)}
            className="h-9 px-3 gap-1.5 text-xs font-semibold rounded-xl border-border/70 hover:bg-muted shadow-sm"
          >
            <Palette className="h-3.5 w-3.5 text-primary" />
            <span>Theme & Colors</span>
          </Button>
        </div>

        {/* Center Card Container with Depth Shadow */}
        <div className="my-auto w-full max-w-md mx-auto py-8">
          <div className="space-y-6">
            <div className="space-y-2">
              <div className="inline-flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-tr from-primary to-orange-500 text-white shadow-lg shadow-primary/25 mb-1">
                <LogIn className="h-6 w-6" />
              </div>
              <h1 className="text-2xl sm:text-3xl font-black tracking-tight text-foreground">
                Sign in to Coachify
              </h1>
              <p className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                Enter your credentials to access your coaching management counter.
              </p>
            </div>

            {recoverySuccessMsg && (
              <div className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-xs font-medium flex items-center gap-2">
                <CheckCircle2 className="h-4 w-4 shrink-0" />
                <span>{recoverySuccessMsg}</span>
              </div>
            )}

            <Card className="rounded-3xl border-border/80 shadow-2xl shadow-primary/5 bg-card/90 backdrop-blur-md overflow-hidden">
              <CardContent className="p-6 sm:p-8">
                <form onSubmit={handleLoginSubmit} className="space-y-4">
                  <div className="space-y-1.5">
                    <label
                      htmlFor="login-user"
                      className="text-xs font-bold text-foreground flex items-center gap-1.5"
                    >
                      <User className="h-3.5 w-3.5 text-primary" />
                      <span>Username</span>
                    </label>
                    <Input
                      id="login-user"
                      placeholder="e.g. owner or cashier1"
                      value={loginUsername}
                      onChange={(e) => {
                        setLoginUsername(e.target.value);
                        if (activeError) {
                          setLocalError(null);
                          clearError();
                        }
                      }}
                      disabled={submitting}
                      autoFocus
                      className="h-11 rounded-xl text-xs sm:text-sm bg-background/80 border-border/80 shadow-inner"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <label
                      htmlFor="login-pass"
                      className="text-xs font-bold text-foreground flex items-center gap-1.5"
                    >
                      <Lock className="h-3.5 w-3.5 text-primary" />
                      <span>Password</span>
                    </label>
                    <Input
                      id="login-pass"
                      type="password"
                      placeholder="Enter your password"
                      value={loginPassword}
                      onChange={(e) => {
                        setLoginPassword(e.target.value);
                        if (activeError) {
                          setLocalError(null);
                          clearError();
                        }
                      }}
                      disabled={submitting}
                      className="h-11 rounded-xl text-xs sm:text-sm bg-background/80 border-border/80 shadow-inner"
                    />
                  </div>

                  {activeError && (
                    <div className="flex items-center gap-2.5 p-3 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">
                      <AlertCircle className="h-4 w-4 shrink-0" />
                      <span>{activeError}</span>
                    </div>
                  )}

                  <Button
                    type="submit"
                    disabled={submitting}
                    className="w-full h-12 rounded-xl text-sm font-bold gap-2 shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/30 transition-all duration-200 mt-2"
                  >
                    <span>{submitting ? "Signing in..." : "Sign In to Counter"}</span>
                    <ArrowRight className="h-4 w-4" />
                  </Button>
                </form>
              </CardContent>
            </Card>

            {/* Disaster Recovery Trigger Button */}
            <div className="text-center pt-1">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setEmergencyOpen(true)}
                className="text-xs text-muted-foreground hover:text-foreground gap-1.5"
              >
                <KeyRound className="h-3.5 w-3.5 text-primary" />
                <span>Disaster Recovery / Unlock Database</span>
              </Button>
            </div>

            {/* Reassuring Role Tags */}
            <div className="flex items-center justify-center gap-2 text-[11px] text-muted-foreground pt-1">
              <Building2 className="h-3.5 w-3.5 text-primary" />
              <span>Multi-user terminal · Owner, Teacher &amp; Cashier support</span>
            </div>
          </div>
        </div>

        {/* Bottom Footer Credits */}
        <div className="flex items-center justify-between text-[11px] text-muted-foreground pt-4 border-t border-border/50">
          <span>Coachify Academic Management OS</span>
          <span className="flex items-center gap-1">
            <Sparkles className="h-3 w-3 text-primary" />
            <span>Dhaka, Bangladesh</span>
          </span>
        </div>
      </div>

      {/* Theme Customizer Dialog */}
      <ThemeCustomizerDialog open={customizerOpen} onOpenChange={setCustomizerOpen} />

      {/* Emergency Recovery Dialog */}
      <EmergencyRecoveryDialog
        open={emergencyOpen}
        onOpenChange={setEmergencyOpen}
        onRecoverySuccess={(msg) => {
          setRecoverySuccessMsg(msg);
          void refreshStatus();
        }}
      />
    </div>
  );
}
