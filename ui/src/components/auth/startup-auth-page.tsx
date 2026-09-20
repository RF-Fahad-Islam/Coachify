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
  Building2,
  CheckCircle2,
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
          {needsSetup ? (
            /* First-Run Bootstrap Owner Form */
            <div className="space-y-6">
              <div className="space-y-2">
                <div className="inline-flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-tr from-primary to-orange-500 text-white shadow-lg shadow-primary/25 mb-1">
                  <ShieldCheck className="h-6 w-6" />
                </div>
                <h1 className="text-2xl sm:text-3xl font-black tracking-tight text-foreground">
                  Welcome to Coachify
                </h1>
                <p className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                  First-run initialization detected. Create the primary Institute Owner
                  account to get started.
                </p>
              </div>

              <Card className="rounded-3xl border-border/80 shadow-2xl shadow-primary/5 bg-card/90 backdrop-blur-md overflow-hidden">
                <CardContent className="p-6 sm:p-8">
                  <form onSubmit={handleBootstrapSubmit} className="space-y-4">
                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-fullname"
                        className="text-xs font-bold text-foreground flex items-center gap-1.5"
                      >
                        <User className="h-3.5 w-3.5 text-primary" />
                        <span>Director / Owner Full Name</span>
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
                        className="h-11 rounded-xl text-xs sm:text-sm bg-background/80 border-border/80 shadow-inner"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-username"
                        className="text-xs font-bold text-foreground flex items-center gap-1.5"
                      >
                        <UserCheck className="h-3.5 w-3.5 text-primary" />
                        <span>Login Username</span>
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
                        className="h-11 rounded-xl text-xs sm:text-sm bg-background/80 border-border/80 shadow-inner"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-password"
                        className="text-xs font-bold text-foreground flex items-center gap-1.5"
                      >
                        <Lock className="h-3.5 w-3.5 text-primary" />
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
                        className="h-11 rounded-xl text-xs sm:text-sm bg-background/80 border-border/80 shadow-inner"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label
                        htmlFor="startup-confirm-password"
                        className="text-xs font-bold text-foreground flex items-center gap-1.5"
                      >
                        <Lock className="h-3.5 w-3.5 text-primary" />
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
                      <span>
                        {submitting ? "Initializing Database..." : "Create Owner Account"}
                      </span>
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </form>
                </CardContent>
              </Card>

              {/* Reassuring Security Highlights */}
              <div className="flex items-center justify-center gap-4 text-[11px] text-muted-foreground pt-1">
                <span className="flex items-center gap-1">
                  <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />
                  <span>Local SQLite Storage</span>
                </span>
                <span className="flex items-center gap-1">
                  <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />
                  <span>Argon2id Encrypted</span>
                </span>
              </div>
            </div>
          ) : (
            /* Standard Staff / Owner Sign In Form */
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

              {/* Reassuring Role Tags */}
              <div className="flex items-center justify-center gap-2 text-[11px] text-muted-foreground pt-1">
                <Building2 className="h-3.5 w-3.5 text-primary" />
                <span>Multi-user terminal · Owner, Teacher & Cashier support</span>
              </div>
            </div>
          )}
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
    </div>
  );
}
