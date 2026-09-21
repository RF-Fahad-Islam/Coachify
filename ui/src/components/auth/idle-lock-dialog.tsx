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
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Lock, ArrowRight, LogOut, AlertCircle } from "lucide-react";

export function IdleLockDialog() {
  const { isIdleLocked, currentUser, unlock, logout, error, clearError } = useAuth();
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);

  if (!isIdleLocked || !currentUser) {
    return null;
  }

  const initials =
    currentUser.display_name
      .split(" ")
      .map((part) => part[0])
      .filter(Boolean)
      .slice(0, 2)
      .join("")
      .toUpperCase() || "U";

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!password.trim()) return;

    setSubmitting(true);
    const success = await unlock(password);
    setSubmitting(false);
    if (success) {
      setPassword("");
    }
  };

  return (
    <Dialog open={isIdleLocked}>
      <DialogContent
        className="sm:max-w-md p-6 border border-border bg-card shadow-xl"
        onEscapeKeyDown={(e) => e.preventDefault()}
        onPointerDownOutside={(e) => e.preventDefault()}
      >
        <DialogHeader className="text-center space-y-2 items-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary-subtle text-primary mb-1">
            <Lock className="h-6 w-6" />
          </div>
          <DialogTitle className="text-xl font-bold tracking-tight text-foreground">
            Session Locked
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground max-w-xs mx-auto">
            You were idle for more than 15 minutes. Enter your password to resume your
            workflow.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col items-center py-4 space-y-4">
          <div className="flex items-center gap-3 p-2.5 rounded-lg bg-muted/50 border border-border/60 w-full">
            <Avatar className="h-10 w-10 border border-border">
              <AvatarFallback className="bg-primary text-primary-foreground font-semibold">
                {initials}
              </AvatarFallback>
            </Avatar>
            <div className="flex flex-col text-left overflow-hidden">
              <span className="text-sm font-semibold text-foreground truncate">
                {currentUser.display_name}
              </span>
              <span className="text-xs text-muted-foreground truncate">
                @{currentUser.username}
              </span>
            </div>
          </div>

          <form onSubmit={handleSubmit} className="w-full space-y-4">
            <div className="space-y-1.5">
              <label
                htmlFor="lock-password"
                className="text-xs font-medium text-foreground"
              >
                Password
              </label>
              <Input
                id="lock-password"
                type="password"
                placeholder="Enter your account password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  if (error) clearError();
                }}
                disabled={submitting}
                autoFocus
                className="h-10 text-sm"
              />
            </div>

            {error && (
              <div className="flex items-center gap-2 p-2.5 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <span>{error}</span>
              </div>
            )}

            <div className="flex flex-col gap-2 pt-1">
              <Button
                type="submit"
                disabled={submitting || !password.trim()}
                className="w-full h-10 gap-2 font-medium"
              >
                <span>{submitting ? "Unlocking..." : "Unlock Session"}</span>
                <ArrowRight className="h-4 w-4" />
              </Button>

              <Button
                type="button"
                variant="ghost"
                onClick={() => void logout()}
                disabled={submitting}
                className="w-full h-9 text-xs text-muted-foreground hover:text-foreground gap-1.5"
              >
                <LogOut className="h-3.5 w-3.5" />
                <span>Log out or Switch Account</span>
              </Button>
            </div>
          </form>
        </div>
      </DialogContent>
    </Dialog>
  );
}
