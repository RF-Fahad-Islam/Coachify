import { useEffect, useState } from "react";
import {
  Search,
  Palette,
  ChevronDown,
  CheckCircle2,
  Lock,
  LogOut,
  Menu,
  Sparkles,
  Languages,
  HelpCircle,
  Smartphone,
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
import { rpc } from "@/gen/rpc";
import { ResilienceStatusBadge } from "@/components/layout/resilience-status-badge";
import { WorkstationPill } from "@/components/layout/workstation-pill";
import { DateRangeFilter } from "@/components/layout/date-range-filter";
import { NotificationsPopover } from "@/components/layout/notifications-popover";
import { useLocale } from "@/lib/i18n";

interface TopBarProps {
  onOpenThemeCustomizer: () => void;
  onSearchClick?: () => void;
  onToggleMobileSidebar?: () => void;
  onOpenCopilot?: () => void;
  onOpenHelp?: () => void;
  onNavigateTab?: (tabId: string) => void;
  selectedDateRange?: string;
  onDateRangeChange?: (rangeId: string) => void;
}

export function TopBar({
  onOpenThemeCustomizer,
  onSearchClick,
  onToggleMobileSidebar,
  onOpenCopilot,
  onOpenHelp,
  onNavigateTab,
  selectedDateRange,
  onDateRangeChange,
}: TopBarProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [isDemoActive, setIsDemoActive] = useState<boolean>(false);
  const [isReverting, setIsReverting] = useState<boolean>(false);
  const { currentUser, roles, lockManually, logout } = useAuth();
  const { locale, toggleLocale, t } = useLocale();

  useEffect(() => {
    let mounted = true;
    const checkDemoStatus = async () => {
      try {
        const res = await rpc("generator.get_status", {});
        if (mounted && res.ok && res.data) {
          setIsDemoActive(res.data.is_demo_active);
        }
      } catch {
        // Ignore
      }
    };
    void checkDemoStatus();
    return () => {
      mounted = false;
    };
  }, []);

  const handleRevertToProduction = async () => {
    setIsReverting(true);
    try {
      const res = await rpc("generator.switch_to_production_database", {});
      if (res.ok) {
        if (typeof window !== "undefined") {
          window.location.reload();
        }
      }
    } catch {
      // Ignore
    } finally {
      setIsReverting(false);
    }
  };

  // Listen for Ctrl+K shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onSearchClick?.();
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "j") {
        e.preventDefault();
        onOpenCopilot?.();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onSearchClick, onOpenCopilot]);

  const initials = currentUser
    ? currentUser.display_name
        .split(" ")
        .map((part) => part[0])
        .filter(Boolean)
        .slice(0, 2)
        .join("")
        .toUpperCase() || "U"
    : "RM";

  const primaryRole = roles[0] || "Owner";
  const displayUserName = currentUser?.display_name || "Rohan Mehta";

  return (
    <header className="h-16 shrink-0 border-b border-border/80 bg-card/90 backdrop-blur-md px-3 sm:px-6 flex items-center justify-between sticky top-0 z-20 gap-3 shadow-xs">
      {/* Left: Mobile Toggle & Global Search */}
      <div className="flex items-center gap-2 flex-1 max-w-[220px] sm:max-w-xs md:max-w-md">
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggleMobileSidebar}
          className="lg:hidden text-muted-foreground hover:text-foreground hover:bg-muted shrink-0 cursor-pointer h-9 w-9 rounded-xl"
          aria-label="Toggle navigation menu"
        >
          <Menu className="h-5 w-5" />
        </Button>
        <div className="relative flex-1">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder={
              locale === "bn"
                ? t("topbar.searchPlaceholder")
                : "Search students, payments, classes... (Ctrl+K)"
            }
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onClick={onSearchClick}
            className="w-full h-10 pl-10 pr-8 sm:pr-14 text-xs sm:text-sm rounded-xl border border-border/80 bg-muted/40 placeholder:text-muted-foreground/70 shadow-2xs focus:bg-card focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all"
          />
          <kbd className="hidden sm:inline-block absolute right-2.5 top-1/2 -translate-y-1/2 text-[11px] font-semibold text-muted-foreground/80 bg-card border border-border px-2 py-0.5 rounded-lg shadow-2xs select-none">
            Ctrl K
          </kbd>
        </div>
      </div>

      {/* Middle: Demo Mode Persistent Indicator */}
      {isDemoActive && (
        <div className="hidden sm:flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/15 border border-amber-500/30 text-amber-900 dark:text-amber-200 text-xs font-medium">
          <span className="h-2 w-2 rounded-full bg-amber-500 animate-pulse" />
          <span>Demo Mode</span>
          <Button
            variant="ghost"
            size="sm"
            disabled={isReverting}
            onClick={handleRevertToProduction}
            className="h-6 px-2 text-[11px] hover:bg-amber-500/20 text-amber-900 dark:text-amber-200 cursor-pointer rounded-lg"
          >
            {isReverting ? "Reverting..." : "Revert"}
          </Button>
        </div>
      )}

      {/* Right: Date Range, Quick Tools, Status Pills, Notifications, and User Profile */}
      <div className="flex items-center gap-2 sm:gap-3">
        {/* Date Filter */}
        <DateRangeFilter value={selectedDateRange} onRangeChange={onDateRangeChange} />

        {/* Quick Tools Segmented Bar */}
        <div className="hidden sm:flex items-center gap-0.5 p-1 rounded-2xl bg-muted/50 border border-border/60 shadow-2xs">
          {/* 1-Click Language Switcher (EN | বাংলা) */}
          <Button
            variant="ghost"
            size="sm"
            onClick={toggleLocale}
            data-testid="language-toggle-btn"
            title="Switch Language (English / বাংলা)"
            className="h-8 px-2.5 text-xs font-semibold gap-1.5 rounded-xl text-foreground hover:bg-card cursor-pointer shadow-2xs"
          >
            <Languages className="h-3.5 w-3.5 text-primary" />
            <span>{locale === "en" ? "বাংলা" : "EN"}</span>
          </Button>

          {/* AI Copilot Quick Trigger */}
          <Button
            variant="ghost"
            size="icon"
            onClick={onOpenCopilot}
            title="Coachify Copilot (Ctrl+J)"
            className="text-primary hover:text-primary hover:bg-card cursor-pointer h-8 w-8 rounded-xl"
            aria-label="Open AI Copilot"
          >
            <Sparkles className="h-4 w-4 text-primary" />
          </Button>

          {/* Theme Customizer Quick Trigger */}
          <Button
            variant="ghost"
            size="icon"
            onClick={onOpenThemeCustomizer}
            title="Customize Theme & Appearance"
            className="text-muted-foreground hover:text-foreground hover:bg-card cursor-pointer h-8 w-8 rounded-xl"
          >
            <Palette className="h-4 w-4" />
          </Button>

          {/* Offline Help & Documentation Quick Trigger */}
          <Button
            variant="ghost"
            size="icon"
            onClick={onOpenHelp}
            data-testid="open-help-btn"
            title="Help & Offline Guides (F1)"
            aria-label="Open Help & Documentation"
            className="text-muted-foreground hover:text-foreground hover:bg-card cursor-pointer h-8 w-8 rounded-xl"
          >
            <HelpCircle className="h-4 w-4" />
          </Button>

          {/* Mobile Companion Trigger */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => {
              const url = new URL(window.location.href);
              url.searchParams.set("mode", "mobile");
              window.location.href = url.toString();
            }}
            data-testid="open-mobile-companion-btn"
            title="Switch to Mobile Companion View"
            aria-label="Open Mobile Companion"
            className="text-emerald-500 hover:text-emerald-400 hover:bg-card cursor-pointer h-8 w-8 rounded-xl"
          >
            <Smartphone className="h-4 w-4" />
          </Button>
        </div>

        {/* Mobile-only Language Toggle */}
        <Button
          variant="outline"
          size="sm"
          onClick={toggleLocale}
          className="sm:hidden h-8 px-2 text-xs font-semibold gap-1 rounded-xl"
        >
          <Languages className="h-3.5 w-3.5 text-primary" />
          <span>{locale === "en" ? "বাংলা" : "EN"}</span>
        </Button>

        {/* Resilience Status Badge */}
        <ResilienceStatusBadge />

        {/* Workstation Client Status Pill */}
        <WorkstationPill />

        {/* Interactive Notifications Popover */}
        <NotificationsPopover onNavigateTab={onNavigateTab} />

        <div className="h-6 w-px bg-border/80 mx-0.5" />

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
            <DropdownMenuContent
              align="end"
              side="bottom"
              sideOffset={8}
              collisionPadding={16}
              className="w-52"
            >
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
              <AvatarFallback className="bg-primary/10 text-primary font-semibold">
                {initials}
              </AvatarFallback>
            </Avatar>
            <div className="hidden sm:flex flex-col text-left">
              <span className="text-xs font-semibold text-foreground leading-none">
                {displayUserName}
              </span>
              <span className="text-[11px] text-muted-foreground leading-none mt-1">
                {primaryRole}
              </span>
            </div>
          </div>
        )}
      </div>
    </header>
  );
}
