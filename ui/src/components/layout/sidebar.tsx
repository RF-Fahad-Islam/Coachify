import { useEffect, useMemo } from "react";
import {
  LayoutDashboard,
  Users,
  CalendarCheck,
  CreditCard,
  FileText,
  Layers,
  GraduationCap,
  BarChart3,
  Settings,
  Sparkles,
  BookOpen,
  MessageSquare,
  ShieldCheck,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/auth-context";
import { useLocale } from "@/lib/i18n";

const NAV_TRANSLATION_KEYS: Record<string, string> = {
  dashboard: "nav.dashboard",
  students: "nav.students",
  attendance: "nav.attendance",
  communication: "nav.communication",
  payments: "nav.payments",
  exams: "nav.exams",
  classes: "nav.academic",
  teachers: "nav.staff",
  intelligence: "nav.intelligence",
  integrity: "nav.integrity",
  reports: "nav.reports",
  settings: "nav.settings",
};

export interface NavItem {
  id: string;
  label: string;
  icon: typeof LayoutDashboard;
  badge?: string | number;
  permissions?: string[];
}

export const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  {
    id: "students",
    label: "Students",
    icon: Users,
    badge: "1.2k",
    permissions: ["students.view"],
  },
  {
    id: "attendance",
    label: "Attendance",
    icon: CalendarCheck,
    permissions: ["attendance.mark"],
  },
  {
    id: "communication",
    label: "Messaging",
    icon: MessageSquare,
    permissions: ["comms.view"],
  },
  {
    id: "payments",
    label: "Payments",
    icon: CreditCard,
    badge: "Due",
    permissions: ["payments.view"],
  },
  { id: "exams", label: "Exams", icon: FileText, permissions: ["exam.view"] },
  { id: "classes", label: "Classes", icon: Layers, permissions: ["academics.view"] },
  {
    id: "teachers",
    label: "Teachers",
    icon: GraduationCap,
    permissions: ["academics.view"],
  },
  {
    id: "intelligence",
    label: "Intelligence",
    icon: Sparkles,
    permissions: ["intelligence.view"],
  },
  {
    id: "integrity",
    label: "Integrity",
    icon: ShieldCheck,
    permissions: ["integrity.view"],
  },
  {
    id: "reports",
    label: "Reports",
    icon: BarChart3,
    permissions: ["reports.view", "reports.financial", "reports.academic"],
  },
  {
    id: "settings",
    label: "Settings",
    icon: Settings,
    permissions: ["system.settings", "users.manage", "roles.manage"],
  },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (tabId: string) => void;
  onOpenThemeCustomizer: () => void;
  mobileOpen?: boolean;
  onCloseMobile?: () => void;
}

export function Sidebar({
  activeTab,
  onTabChange,
  onOpenThemeCustomizer,
  mobileOpen = false,
  onCloseMobile,
}: SidebarProps) {
  const { hasPermission, isOwner } = useAuth();
  const { locale, t } = useLocale();

  const visibleNavItems = useMemo(() => {
    return NAV_ITEMS.filter((item) => {
      if (!item.permissions || item.permissions.length === 0) return true;
      if (isOwner()) return true;
      return item.permissions.some((p) => hasPermission(p));
    });
  }, [hasPermission, isOwner]);

  useEffect(() => {
    if (!mobileOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onCloseMobile?.();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [mobileOpen, onCloseMobile]);

  const renderContent = (isMobileDrawer: boolean) => (
    <>
      {/* Top: Logo & Branding */}
      <div>
        <div className="h-16 flex items-center justify-between px-6 border-b border-border/70">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-primary text-primary-foreground shadow-xs">
              <BookOpen className="h-5 w-5 stroke-[2.5]" />
            </div>
            <div className="flex flex-col">
              <span className="font-bold text-base leading-tight tracking-tight text-foreground">
                Coachify
              </span>
              <span className="text-xs font-medium text-muted-foreground">
                Coaching Management System
              </span>
            </div>
          </div>
          {isMobileDrawer && (
            <button
              type="button"
              onClick={onCloseMobile}
              className="lg:hidden p-1.5 rounded-[var(--radius-sm)] text-muted-foreground hover:text-foreground hover:bg-muted cursor-pointer"
              aria-label="Close navigation menu"
            >
              <X className="h-5 w-5" />
            </button>
          )}
        </div>

        {/* Navigation Items */}
        <nav className="p-3 space-y-1 mt-2">
          {visibleNavItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            const displayLabel =
              locale === "bn" && NAV_TRANSLATION_KEYS[item.id]
                ? t(NAV_TRANSLATION_KEYS[item.id])
                : item.label;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => {
                  onTabChange(item.id);
                  if (isMobileDrawer) onCloseMobile?.();
                }}
                className={cn(
                  "w-full flex items-center justify-between px-3.5 py-2.5 rounded-[var(--radius)] text-xs sm:text-sm font-medium transition-all cursor-pointer min-h-[40px]",
                  isActive
                    ? "bg-primary-subtle text-primary border-l-4 border-primary font-semibold shadow-xs"
                    : "text-muted-foreground hover:text-foreground hover:bg-muted/70"
                )}
              >
                <div className="flex items-center gap-3">
                  <Icon
                    className={cn(
                      "h-4 w-4 shrink-0",
                      isActive ? "text-primary" : "text-muted-foreground"
                    )}
                  />
                  <span>{displayLabel}</span>
                </div>
                {item.badge && (
                  <span
                    className={cn(
                      "px-2 py-0.5 text-[11px] font-semibold rounded-full",
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground"
                    )}
                  >
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Bottom: Motivational & Customizer Card */}
      <div className="p-4 border-t border-border/70">
        <div className="relative overflow-hidden rounded-[var(--radius-lg)] border border-primary-border/60 bg-gradient-to-br from-primary-subtle via-primary-subtle/70 to-card p-4 shadow-xs hover:shadow-md transition-all duration-200">
          <div className="flex items-center gap-2 text-primary font-semibold text-xs mb-1">
            <Sparkles className="h-4 w-4" />
            <span>Empowering Futures</span>
          </div>
          <p className="text-xs text-muted-foreground leading-relaxed mb-3">
            Manage today, build tomorrow. Personalize your workspace styling and density.
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={onOpenThemeCustomizer}
            className="w-full text-xs gap-1.5 border-primary/30 text-primary hover:bg-primary-subtle cursor-pointer"
          >
            <Sparkles className="h-3.5 w-3.5 text-primary" />
            Customize Theme
          </Button>
        </div>
      </div>
    </>
  );

  return (
    <>
      {/* Desktop Persistent Sidebar */}
      <aside className="w-64 shrink-0 hidden lg:flex flex-col justify-between border-r border-border bg-sidebar text-sidebar-foreground select-none h-screen sticky top-0">
        {renderContent(false)}
      </aside>

      {/* Mobile / Tablet Drawer Overlay & Off-canvas Menu */}
      {mobileOpen && (
        <div className="lg:hidden fixed inset-0 z-50 flex">
          {/* Backdrop overlay */}
          <div
            className="fixed inset-0 bg-black/50 backdrop-blur-xs transition-opacity"
            onClick={onCloseMobile}
            aria-hidden="true"
          />

          {/* Slide-over Drawer panel */}
          <aside className="relative z-50 w-72 max-w-[85vw] flex flex-col justify-between border-r border-border bg-sidebar text-sidebar-foreground select-none h-full shadow-2xl animate-in slide-in-from-left duration-200">
            {renderContent(true)}
          </aside>
        </div>
      )}
    </>
  );
}
