import { useState, type FormEvent } from "react";
import { useAuth } from "@/contexts/auth-context";
import { OnboardingCarousel } from "./onboarding-carousel";
import { ThemeCustomizerDialog } from "@/components/theme/theme-customizer-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import {
  ShieldCheck,
  LogIn,
  ArrowRight,
  AlertCircle,
  Palette,
  Sparkles,
  Lock,
  User,
  UserCheck,
} from "lucide-react";

export function StartupAuthPage() {
  const { needsSetup, bootstrapOwner, login, error, clearError } = useAuth();
  const [customizerOpen, setCustomizerOpen] = useState(false);

  // Bootstrap owner form state
  const [displayName, setDisplayName] = useState("");
  const [bootstrapUsername, setBootstrapUsername] = useState("owner");
  const [bootstrapPassword, setBootstrapPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  // Login form state
  const [loginUsername, setLoginUsername] = useState("");
  const [loginPassword, setLoginPassword] = useState("");

  const [submitting, setSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  // Owner bootstrap submit handler
  const handleBootstrapSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (!displayName.trim() || !bootstrapUsername.trim() || !bootstrapPassword) {
      setLocalError("Please fill in all required fields.");
      return;
    }

    if (bootstrapPassword.length < 8) {
      setLocalError("Password must be at least 8 characters long.");
      return;
    }

    if (bootstrapPassword !== confirmPassword) {
      setLocalError("Passwords do not match.");
      return;
    }

    setSubmitting(true);
    await bootstrapOwner(bootstrapUsername.trim(), displayName.trim(), bootstrapPassword);
    setSubmitting(false);
  };

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

  const activeError = localError || error;

  return (
    <div className="min-h-screen w-full flex flex-col md:flex-row bg-background text-foreground antialiased select-none">
      {/* Left Column: Onboarding & Brand Showcase */}
      <div className="w-full md:w-1/2 lg:w-3/5 bg-gradient-to-br from-primary-subtle/30 via-background to-background border-b md:border-b-0 md:border-r border-border/80 flex flex-col justify-between">
        <OnboardingCarousel />
      </div>

      {/* Right Column: Authentication Card Container */}
      <div className="w-full md:w-1/2 lg:w-2/5 flex flex-col justify-between p-6 sm:p-10 lg:p-12 relative">
        {/* Top bar controls on right side */}
        <div className="flex items-center justify-end gap-2 w-full">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setCustomizerOpen(true)}
            className="h-8 gap-1.5 text-xs text-muted-foreground hover:text-foreground"
          >
            <Palette className="h-3.5 w-3.5 text-primary" />
            <span>Appearance</span>
          </Button>
        </div>

        {/* Center Card Container */}
        <div className="my-auto w-full max-w-md mx-auto py-8">
          {needsSetup ? (
            /* First-Run Bootstrap Owner Form */
            <div className="space-y-6">
              <div className="space-y-2">
                <div className="inline-flex h-11 w-11 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20 mb-1">
                  <ShieldCheck className="h-6 w-6" />
                </div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Welcome to Coachify
                </h1>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  First-run initialization detected. Set up the primary Administrator /
                  Owner account for your coaching center.
                </p>
              </div>

              <Card className="border-border/80 shadow-sm bg-card/60 backdrop-blur-sm">
                <CardContent className="p-5">
                  <form onSubmit={handleBootstrapSubmit} className="space-y-4">
                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-fullname"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <User className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>Full Name</span>
                      </label>
                      <Input
                        id="startup-fullname"
                        placeholder="e.g. Masud Rana"
                        value={displayName}
                        onChange={(e) => {
                          setDisplayName(e.target.value);
                          if (activeError) {
                            setLocalError(null);
                            clearError();
                          }
                        }}
                        disabled={submitting}
                        autoFocus
                        className="h-9 text-xs"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-username"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <UserCheck className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>Username</span>
                      </label>
                      <Input
                        id="startup-username"
                        placeholder="e.g. owner"
                        value={bootstrapUsername}
                        onChange={(e) => {
                          setBootstrapUsername(e.target.value);
                          if (activeError) {
                            setLocalError(null);
                            clearError();
                          }
                        }}
                        disabled={submitting}
                        className="h-9 text-xs"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-password"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <Lock className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>Password (min 8 characters)</span>
                      </label>
                      <Input
                        id="startup-password"
                        type="password"
                        placeholder="Enter secure password"
                        value={bootstrapPassword}
                        onChange={(e) => {
                          setBootstrapPassword(e.target.value);
                          if (activeError) {
                            setLocalError(null);
                            clearError();
                          }
                        }}
                        disabled={submitting}
                        className="h-9 text-xs"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-confirm-password"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <Lock className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>Confirm Password</span>
                      </label>
                      <Input
                        id="startup-confirm-password"
                        type="password"
                        placeholder="Re-enter password"
                        value={confirmPassword}
                        onChange={(e) => {
                          setConfirmPassword(e.target.value);
                          if (activeError) {
                            setLocalError(null);
                            clearError();
                          }
                        }}
                        disabled={submitting}
                        className="h-9 text-xs"
                      />
                    </div>

                    {activeError && (
                      <div className="flex items-center gap-2 p-2.5 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                        <AlertCircle className="h-4 w-4 shrink-0" />
                        <span>{activeError}</span>
                      </div>
                    )}

                    <Button
                      type="submit"
                      disabled={submitting}
                      className="w-full h-10 gap-2 font-medium mt-2"
                    >
                      <span>
                        {submitting ? "Initializing Database..." : "Create Owner Account"}
                      </span>
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </form>
                </CardContent>
              </Card>

              <div className="text-center">
                <p className="text-[11px] text-muted-foreground">
                  Database is hosted locally on this machine · Encrypted with Argon2id
                </p>
              </div>
            </div>
          ) : (
            /* Standard Staff / Owner Sign In Form */
            <div className="space-y-6">
              <div className="space-y-2">
                <div className="inline-flex h-11 w-11 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20 mb-1">
                  <LogIn className="h-6 w-6" />
                </div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Sign in to Coachify
                </h1>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Enter your credentials to access the coaching management counter.
                </p>
              </div>

              <Card className="border-border/80 shadow-sm bg-card/60 backdrop-blur-sm">
                <CardContent className="p-5">
                  <form onSubmit={handleLoginSubmit} className="space-y-4">
                    <div className="space-y-1.5">
                      <label
                        htmlFor="login-user"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <User className="h-3.5 w-3.5 text-muted-foreground" />
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
                        className="h-9 text-xs"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="login-pass"
                        className="text-xs font-semibold text-foreground flex items-center gap-1.5"
                      >
                        <Lock className="h-3.5 w-3.5 text-muted-foreground" />
                        <span>Password</span>
                      </label>
                      <Input
                        id="login-pass"
                        type="password"
                        placeholder="Enter password"
                        value={loginPassword}
                        onChange={(e) => {
                          setLoginPassword(e.target.value);
                          if (activeError) {
                            setLocalError(null);
                            clearError();
                          }
                        }}
                        disabled={submitting}
                        className="h-9 text-xs"
                      />
                    </div>

                    {activeError && (
                      <div className="flex items-center gap-2 p-2.5 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                        <AlertCircle className="h-4 w-4 shrink-0" />
                        <span>{activeError}</span>
                      </div>
                    )}

                    <Button
                      type="submit"
                      disabled={submitting}
                      className="w-full h-10 gap-2 font-medium mt-2"
                    >
                      <span>{submitting ? "Signing in..." : "Sign In to Counter"}</span>
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </form>
                </CardContent>
              </Card>

              <div className="text-center">
                <p className="text-[11px] text-muted-foreground">
                  Multi-account workstation · Session automatically idle-locks after 15m
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Bottom Footer Credits */}
        <div className="flex items-center justify-between text-[11px] text-muted-foreground pt-4 border-t border-border/40">
          <span>Coachify Academic Management OS</span>
          <span className="flex items-center gap-1">
            <Sparkles className="h-3 w-3 text-primary" />
            <span>Dhaka, Bangladesh</span>
          </span>
        </div>
      </div>

      {/* Theme Customizer Dialog (Accessible on startup) */}
      <ThemeCustomizerDialog open={customizerOpen} onOpenChange={setCustomizerOpen} />
    </div>
  );
}
