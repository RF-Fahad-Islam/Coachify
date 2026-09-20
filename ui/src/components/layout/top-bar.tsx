import { useEffect, useState } from "react";
import { Search, Calendar, Bell, Palette, ChevronDown, CheckCircle2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";

interface TopBarProps {
  onOpenThemeCustomizer: () => void;
  onSearchClick?: () => void;
}

export function TopBar({ onOpenThemeCustomizer, onSearchClick }: TopBarProps) {
  const [searchQuery, setSearchQuery] = useState("");

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
        <div className="flex items-center gap-3 pl-1">
          <Avatar className="h-8 w-8">
            <AvatarFallback>RM</AvatarFallback>
          </Avatar>
          <div className="hidden sm:flex flex-col text-left">
            <div className="flex items-center gap-1.5">
              <span className="text-xs font-semibold text-foreground leading-none">
                Rohan Mehta
              </span>
              <CheckCircle2 className="h-3 w-3 text-primary" />
            </div>
            <span className="text-[11px] text-muted-foreground leading-none mt-1">
              Admin
            </span>
          </div>
          <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
        </div>
      </div>
    </header>
  );
}
