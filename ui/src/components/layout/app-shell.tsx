import { useState, type ReactNode } from "react";
import { Sidebar } from "./sidebar";
import { TopBar } from "./top-bar";
import { QuickSearchDialog } from "./quick-search-dialog";
import { ThemeCustomizerDialog } from "@/components/theme/theme-customizer-dialog";

interface AppShellProps {
  children: ReactNode;
  activeTab?: string;
  onTabChange?: (tabId: string) => void;
}

export function AppShell({
  children,
  activeTab = "dashboard",
  onTabChange,
}: AppShellProps) {
  const [currentTab, setCurrentTab] = useState(activeTab);
  const [customizerOpen, setCustomizerOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);

  const handleTabChange = (tabId: string) => {
    setCurrentTab(tabId);
    onTabChange?.(tabId);
  };

  return (
    <div className="flex min-h-screen bg-background text-foreground antialiased selection:bg-primary selection:text-primary-foreground">
      {/* Left Sidebar Navigation */}
      <Sidebar
        activeTab={currentTab}
        onTabChange={handleTabChange}
        onOpenThemeCustomizer={() => setCustomizerOpen(true)}
      />

      {/* Main Workspace Area */}
      <div className="flex-1 flex flex-col min-w-0">
        <TopBar
          onOpenThemeCustomizer={() => setCustomizerOpen(true)}
          onSearchClick={() => setSearchOpen(true)}
        />

        <main className="flex-1 p-6 overflow-y-auto">
          <div className="max-w-7xl mx-auto w-full">{children}</div>
        </main>
      </div>

      {/* Command Search Dialog (Ctrl+K) */}
      <QuickSearchDialog
        open={searchOpen}
        onOpenChange={setSearchOpen}
        onSelectTab={handleTabChange}
        onOpenThemeCustomizer={() => setCustomizerOpen(true)}
      />

      {/* Theme Customizer Dialog */}
      <ThemeCustomizerDialog open={customizerOpen} onOpenChange={setCustomizerOpen} />
    </div>
  );
}
