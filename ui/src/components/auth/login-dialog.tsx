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
import { LogIn, ArrowRight, AlertCircle, BookOpen } from "lucide-react";

export function LoginDialog() {
  const { needsSetup, currentUser, login, error, clearError } = useAuth();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);

  // If setup is needed or user is already logged in, do not show login dialog
  if (needsSetup || currentUser) {
    return null;
  }

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!username.trim() || !password) return;

    setSubmitting(true);
    await login(username.trim(), password);
    setSubmitting(false);
  };

  return (
    <Dialog open={!currentUser && !needsSetup}>
      <DialogContent
        className="sm:max-w-md p-6 border border-border bg-card shadow-xl"
        onEscapeKeyDown={(e) => e.preventDefault()}
        onPointerDownOutside={(e) => e.preventDefault()}
      >
        <DialogHeader className="text-center space-y-2 items-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary-subtle text-primary mb-1">
            <BookOpen className="h-6 w-6" />
          </div>
          <DialogTitle className="text-xl font-bold tracking-tight text-foreground">
            Sign in to Coachify
          </DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground max-w-xs mx-auto">
            Enter your credentials to access the coaching management counter.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          <div className="space-y-1">
            <label
              htmlFor="login-username"
              className="text-xs font-medium text-foreground"
            >
              Username
            </label>
            <Input
              id="login-username"
              placeholder="Username"
              value={username}
              onChange={(e) => {
                setUsername(e.target.value);
                if (error) clearError();
              }}
              disabled={submitting}
              autoFocus
              className="h-9 text-xs"
            />
          </div>

          <div className="space-y-1">
            <label
              htmlFor="login-password"
              className="text-xs font-medium text-foreground"
            >
              Password
            </label>
            <Input
              id="login-password"
              type="password"
              placeholder="Password"
              value={password}
              onChange={(e) => {
                setPassword(e.target.value);
                if (error) clearError();
              }}
              disabled={submitting}
              className="h-9 text-xs"
            />
          </div>

          {error && (
            <div className="flex items-center gap-2 p-2.5 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          <Button
            type="submit"
            disabled={submitting || !username.trim() || !password}
            className="w-full h-10 gap-2 font-medium mt-2"
          >
            <LogIn className="h-4 w-4" />
            <span>{submitting ? "Signing in..." : "Sign in"}</span>
            <ArrowRight className="h-4 w-4" />
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
