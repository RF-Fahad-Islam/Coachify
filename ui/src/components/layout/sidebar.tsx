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
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

export interface NavItem {
  id: string;
  label: string;
  icon: typeof LayoutDashboard;
  badge?: string | number;
}

export const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "students", label: "Students", icon: Users, badge: "1.2k" },
  { id: "attendance", label: "Attendance", icon: CalendarCheck },
  { id: "payments", label: "Payments", icon: CreditCard, badge: "Due" },
  { id: "exams", label: "Exams", icon: FileText },
  { id: "classes", label: "Classes", icon: Layers },
  { id: "teachers", label: "Teachers", icon: GraduationCap },
  { id: "reports", label: "Reports", icon: BarChart3 },
  { id: "settings", label: "Settings", icon: Settings },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (tabId: string) => void;
  onOpenThemeCustomizer: () => void;
}

export function Sidebar({ activeTab, onTabChange, onOpenThemeCustomizer }: SidebarProps) {
  return (
    <aside className="w-64 shrink-0 flex flex-col justify-between border-r border-border bg-sidebar text-sidebar-foreground select-none h-screen sticky top-0">
      {/* Top: Logo & Branding */}
      <div>
        <div className="h-16 flex items-center gap-3 px-6 border-b border-border/70">
          <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-primary text-primary-foreground shadow-xs">
            <BookOpen className="h-5 w-5 stroke-[2.5]" />
          </div>
          <div className="flex flex-col">
            <span className="font-bold text-base leading-tight tracking-tight text-foreground">
              Coachify
            </span>
            <span className="text-[11px] font-medium text-muted-foreground">
              Coaching Management System
            </span>
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="p-3 space-y-1 mt-2">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => onTabChange(item.id)}
                className={cn(
                  "w-full flex items-center justify-between px-3.5 py-2.5 rounded-[var(--radius)] text-sm font-medium transition-all cursor-pointer",
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
                  <span>{item.label}</span>
                </div>
                {item.badge && (
                  <span
                    className={cn(
                      "px-2 py-0.5 text-[10px] font-semibold rounded-full",
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
        <div className="relative overflow-hidden rounded-[var(--radius-lg)] border border-primary-border/60 bg-gradient-to-br from-primary-subtle via-primary-subtle/70 to-card p-4 shadow-xs">
          <div className="flex items-center gap-2 text-primary font-semibold text-xs mb-1">
            <Sparkles className="h-4 w-4" />
            <span>Empowering Futures</span>
          </div>
          <p className="text-[11px] text-muted-foreground leading-relaxed mb-3">
            Manage today, build tomorrow. Personalize your workspace styling and density.
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={onOpenThemeCustomizer}
            className="w-full text-xs gap-1.5 border-primary/30 text-primary hover:bg-primary-subtle"
          >
            <Sparkles className="h-3.5 w-3.5 text-primary" />
            Customize Theme
          </Button>
        </div>
      </div>
    </aside>
  );
}
