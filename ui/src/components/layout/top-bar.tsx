import { useEffect, useState } from "react";
import {
  Search,
  Calendar,
  Bell,
  Palette,
  ChevronDown,
  CheckCircle2,
  Lock,
  LogOut,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useAuth } from "@/contexts/auth-context";

interface TopBarProps {
  onOpenThemeCustomizer: () => void;
  onSearchClick?: () => void;
}

export function TopBar({ onOpenThemeCustomizer, onSearchClick }: TopBarProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const { currentUser, roles, lockManually, logout } = useAuth();

  // Listen for Ctrl+K shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onSearchClick?.();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onSearchClick]);

  const initials = currentUser
    ? currentUser.display_name
        .split(" ")
        .map((part) => part[0])
        .filter(Boolean)
        .slice(0, 2)
        .join("")
        .toUpperCase() || "U"
    : "??";

  const primaryRole = roles[0] || "Staff";

  return (
    <header className="h-16 shrink-0 border-b border-border bg-card/80 backdrop-blur-xs px-6 flex items-center justify-between sticky top-0 z-20">
      {/* Left: Global Search */}
      <div className="flex-1 max-w-md">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search students, payments, classes... (Ctrl+K)"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onClick={onSearchClick}
            className="w-full h-9 pl-9 pr-14 text-xs rounded-[var(--radius)] border border-border bg-muted/40 placeholder:text-muted-foreground/70 focus:bg-card focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary transition-all"
          />
          <kbd className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] font-semibold text-muted-foreground/70 bg-card border border-border px-1.5 py-0.5 rounded shadow-2xs select-none">
            Ctrl K
          </kbd>
        </div>
      </div>

      {/* Right: Date Range, Notifications, Theme, and User Profile */}
      <div className="flex items-center gap-3">
        {/* Date Filter */}
        <Button
          variant="outline"
          size="sm"
          className="hidden md:inline-flex text-xs gap-2 font-normal text-muted-foreground hover:text-foreground"
        >
          <Calendar className="h-3.5 w-3.5 text-primary" />
          <span>Apr 21, 2026 – Apr 27, 2026</span>
          <ChevronDown className="h-3.5 w-3.5 opacity-60" />
        </Button>

        {/* Theme Customizer Quick Trigger */}
        <Button
          variant="ghost"
          size="icon"
          onClick={onOpenThemeCustomizer}
          title="Customize Theme & Appearance"
          className="text-muted-foreground hover:text-foreground hover:bg-muted"
        >
          <Palette className="h-4 w-4" />
        </Button>

        {/* Notifications with Badge */}
        <div className="relative">
          <Button
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:text-foreground hover:bg-muted"
          >
            <Bell className="h-4 w-4" />
          </Button>
          <span className="absolute 1 top-1.5 right-1.5 flex h-4 w-4 items-center justify-center rounded-full bg-primary text-[10px] font-bold text-primary-foreground">
            3
          </span>
        </div>

        <div className="h-6 w-px bg-border mx-1" />

        {/* User Profile */}
        {currentUser ? (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button
                type="button"
                className="flex items-center gap-3 pl-1 text-left cursor-pointer rounded-lg p-1 hover:bg-muted/50 transition-colors focus:outline-none"
              >
                <Avatar className="h-8 w-8">
                  <AvatarFallback className="bg-primary/10 text-primary font-semibold">
                    {initials}
                  </AvatarFallback>
                </Avatar>
                <div className="hidden sm:flex flex-col text-left">
                  <div className="flex items-center gap-1.5">
                    <span className="text-xs font-semibold text-foreground leading-none">
                      {currentUser.display_name}
                    </span>
                    <CheckCircle2 className="h-3 w-3 text-primary" />
                  </div>
                  <span className="text-[11px] text-muted-foreground leading-none mt-1">
                    {primaryRole}
                  </span>
                </div>
                <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              <div className="px-2 py-1.5 text-xs text-muted-foreground">
                Signed in as{" "}
                <strong className="text-foreground font-medium">
                  @{currentUser.username}
                </strong>
              </div>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={lockManually}
                className="gap-2 cursor-pointer text-xs"
              >
                <Lock className="h-3.5 w-3.5" />
                <span>Lock Screen (Idle)</span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => void logout()}
                className="gap-2 cursor-pointer text-xs text-destructive focus:text-destructive"
              >
                <LogOut className="h-3.5 w-3.5" />
                <span>Log out</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ) : (
          <div className="flex items-center gap-2 pl-1">
            <Avatar className="h-8 w-8">
              <AvatarFallback className="bg-muted text-muted-foreground">
                --
              </AvatarFallback>
            </Avatar>
            <span className="text-xs text-muted-foreground">Guest</span>
          </div>
        )}
      </div>
    </header>
  );
}
