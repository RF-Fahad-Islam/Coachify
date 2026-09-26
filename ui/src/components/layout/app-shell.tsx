import { useState, useEffect, type ReactNode } from "react";
import { AlertTriangle, X } from "lucide-react";
import { Sidebar } from "./sidebar";
import { TopBar } from "./top-bar";
import { CommandPalette } from "./command-palette";
import { ThemeCustomizerDialog } from "@/components/theme/theme-customizer-dialog";
import { CopilotDrawer } from "@/components/ai/copilot-drawer";
import { HelpDialog } from "@/components/help/help-dialog";
import { ReadOnlyBanner } from "@/components/settings/license-settings-card";
import { ReconnectionBanner } from "@/components/layout/reconnection-banner";
import { WorkstationLockoutOverlay } from "@/components/auth/workstation-lockout-overlay";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/auth-context";

interface AppShellProps {
  children: ReactNode;
  activeTab?: string;
  onTabChange?: (tabId: string) => void;
  onSelectStudent?: (studentId: string) => void;
}

export function AppShell({
  children,
  activeTab = "dashboard",
  onTabChange,
  onSelectStudent,
}: AppShellProps) {
  const [currentTab, setCurrentTab] = useState(activeTab);
  const [customizerOpen, setCustomizerOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const [copilotOpen, setCopilotOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);
  const { previewRole, setPreviewRole } = useAuth();

  const handleTabChange = (tabId: string) => {
    setCurrentTab(tabId);
    onTabChange?.(tabId);
    setMobileSidebarOpen(false);
  };

  // Global Ctrl+J shortcut listener for AI Copilot & F1 for Help
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "j") {
        e.preventDefault();
        setCopilotOpen((prev) => !prev);
      }
      if (e.key === "F1") {
        e.preventDefault();
        setHelpOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="flex min-h-screen bg-background text-foreground antialiased selection:bg-primary selection:text-primary-foreground">
      {/* Sidebar Navigation (Desktop Rail & Mobile Drawer) */}
      <Sidebar
        activeTab={currentTab}
        onTabChange={handleTabChange}
        onOpenThemeCustomizer={() => setCustomizerOpen(true)}
        mobileOpen={mobileSidebarOpen}
        onCloseMobile={() => setMobileSidebarOpen(false)}
      />

      {/* Main Workspace Area */}
      <div className="flex-1 flex flex-col min-w-0">
        <ReconnectionBanner />
        <TopBar
          onOpenThemeCustomizer={() => setCustomizerOpen(true)}
          onSearchClick={() => setSearchOpen(true)}
          onToggleMobileSidebar={() => setMobileSidebarOpen((prev) => !prev)}
          onOpenCopilot={() => setCopilotOpen(true)}
          onOpenHelp={() => setHelpOpen(true)}
        />

        <ReadOnlyBanner onManageLicense={() => handleTabChange("settings")} />

        {/* Spec 0029 AC-6: Amber Preview Mode Banner */}
        {previewRole && (
          <aside
            aria-label="Role Preview Active"
            className="bg-amber-500/15 border-b border-amber-500/30 text-amber-900 dark:text-amber-200 px-4 py-2 flex items-center justify-between text-sm font-medium z-10 transition-all"
          >
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
              <span>
                Zero Training Mode: Viewing Coachify as <strong>{previewRole}</strong>.
              </span>
            </div>
            <Button
              size="sm"
              variant="outline"
              className="h-7 text-xs border-amber-500/40 hover:bg-amber-500/20 text-amber-900 dark:text-amber-200 cursor-pointer"
              onClick={() => setPreviewRole(null)}
            >
              <X className="h-3 w-3 mr-1" />
              Exit Preview
            </Button>
          </aside>
        )}

        <main className="flex-1 p-3 sm:p-4 md:p-6 lg:p-8 overflow-y-auto">
          <div className="max-w-7xl mx-auto w-full">{children}</div>
        </main>
      </div>

      {/* Global Command Palette (Ctrl+K and Chord Shortcuts) */}
      <CommandPalette
        open={searchOpen}
        onOpenChange={setSearchOpen}
        onSelectTab={handleTabChange}
        onOpenThemeCustomizer={() => setCustomizerOpen(true)}
        onSelectStudent={onSelectStudent}
        onOpenCopilot={() => setCopilotOpen(true)}
      />

      {/* Theme Customizer Dialog */}
      <ThemeCustomizerDialog open={customizerOpen} onOpenChange={setCustomizerOpen} />

      {/* Offline Help & Documentation Dialog */}
      <HelpDialog open={helpOpen} onOpenChange={setHelpOpen} />

      {/* AI Copilot Drawer (Ctrl+J) */}
      <CopilotDrawer
        open={copilotOpen}
        onClose={() => setCopilotOpen(false)}
        onOpenSettings={() => handleTabChange("settings")}
      />

      {/* Workstation Severance Lockout Overlay */}
      <WorkstationLockoutOverlay />
    </div>
  );
}
