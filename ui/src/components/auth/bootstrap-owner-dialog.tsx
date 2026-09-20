import { useState, type FormEvent } from "react";
import { useAuth } from "@/contexts/auth-context";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ShieldCheck, ArrowRight, AlertCircle } from "lucide-react";

export function BootstrapOwnerDialog() {
  const { needsSetup, currentUser, bootstrapOwner, error, clearError } = useAuth();
  const [username, setUsername] = useState("owner");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  if (!needsSetup || currentUser) {
    return null;
  }

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (!username.trim() || !displayName.trim() || !password) {
      setLocalError("Please fill in all required fields.");
      return;
    }

    if (password.length < 8) {
      setLocalError("Password must be at least 8 characters long.");
      return;
    }

    if (password !== confirmPassword) {
      setLocalError("Passwords do not match.");
      return;
    }

    setSubmitting(true);
    await bootstrapOwner(username.trim(), displayName.trim(), password);
    setSubmitting(false);
  };

  const activeError = localError || error;

  return (
    <Dialog open={needsSetup && !currentUser}>
      <DialogContent
        className="sm:max-w-md p-6 border border-border bg-card shadow-xl"
        onEscapeKeyDown={(e) => e.preventDefault()}
        onPointerDownOutside={(e) => e.preventDefault()}
      >
        <DialogHeader className="text-center space-y-2 items-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary-subtle text-primary mb-1">
            <ShieldCheck className="h-6 w-6" />
          </div>
          <DialogTitle className="text-xl font-bold tracking-tight text-foreground">
            Welcome to Coachify
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground max-w-xs mx-auto">
            Set up the primary Administrator / Owner account for your coaching center.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          <div className="space-y-1">
            <label
              htmlFor="bootstrap-fullname"
              className="text-xs font-medium text-foreground"
            >
              Full Name
            </label>
            <Input
              id="bootstrap-fullname"
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

          <div className="space-y-1">
            <label
              htmlFor="bootstrap-username"
              className="text-xs font-medium text-foreground"
            >
              Username
            </label>
            <Input
              id="bootstrap-username"
              placeholder="e.g. owner"
              value={username}
              onChange={(e) => {
                setUsername(e.target.value);
                if (activeError) {
                  setLocalError(null);
                  clearError();
                }
              }}
              disabled={submitting}
              className="h-9 text-xs"
            />
          </div>

          <div className="space-y-1">
            <label
              htmlFor="bootstrap-password"
              className="text-xs font-medium text-foreground"
            >
              Password (minimum 8 characters)
            </label>
            <Input
              id="bootstrap-password"
              type="password"
              placeholder="Enter secure password"
              value={password}
              onChange={(e) => {
                setPassword(e.target.value);
                if (activeError) {
                  setLocalError(null);
                  clearError();
                }
              }}
              disabled={submitting}
              className="h-9 text-xs"
            />
          </div>

          <div className="space-y-1">
            <label
              htmlFor="bootstrap-confirm"
              className="text-xs font-medium text-foreground"
            >
              Confirm Password
            </label>
            <Input
              id="bootstrap-confirm"
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
            <span>{submitting ? "Initializing..." : "Create Owner Account"}</span>
            <ArrowRight className="h-4 w-4" />
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
