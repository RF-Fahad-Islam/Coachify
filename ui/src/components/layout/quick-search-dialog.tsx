import { useState } from "react";
import {
  LayoutDashboard,
  Users,
  CalendarCheck,
  CreditCard,
  FileText,
  BarChart3,
  Settings,
  Plus,
  Palette,
} from "lucide-react";
import {
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command";

export interface QuickSearchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectTab: (tabId: string) => void;
  onOpenThemeCustomizer: () => void;
}

export function QuickSearchDialog({
  open,
  onOpenChange,
  onSelectTab,
  onOpenThemeCustomizer,
}: QuickSearchDialogProps) {
  const [query, setQuery] = useState("");

  const handleSelect = (action: () => void) => {
    action();
    onOpenChange(false);
    setQuery("");
  };

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, shortcut: "G D" },
    { id: "students", label: "Students Directory", icon: Users, shortcut: "G S" },
    { id: "attendance", label: "Batch Attendance", icon: CalendarCheck, shortcut: "G A" },
    { id: "payments", label: "Finance & Fee Ledger", icon: CreditCard, shortcut: "G F" },
    { id: "exams", label: "Exams & Mark Entry", icon: FileText, shortcut: "G E" },
    { id: "reports", label: "Analytical Reports", icon: BarChart3, shortcut: "G R" },
    { id: "settings", label: "Institute Settings", icon: Settings, shortcut: "G T" },
  ];

  const filteredNav = navItems.filter((item) =>
    item.label.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput
        placeholder="Type a command or search modules... (e.g. Students, Fee, Theme)"
        value={query}
        onValueChange={setQuery}
      />
      <CommandList>
        {filteredNav.length === 0 && (
          <CommandEmpty>No matching commands found.</CommandEmpty>
        )}

        {filteredNav.length > 0 && (
          <CommandGroup heading="Navigation">
            {filteredNav.map((item) => {
              const Icon = item.icon;
              return (
                <CommandItem
                  key={item.id}
                  onSelect={() => handleSelect(() => onSelectTab(item.id))}
                >
                  <Icon className="h-4 w-4 text-primary" />
                  <span>{item.label}</span>
                  <CommandShortcut>{item.shortcut}</CommandShortcut>
                </CommandItem>
              );
            })}
          </CommandGroup>
        )}

        <CommandGroup heading="Actions">
          <CommandItem
            onSelect={() =>
              handleSelect(() => {
                onSelectTab("students");
              })
            }
          >
            <Plus className="h-4 w-4 text-emerald-500" />
            <span>Enroll New Student</span>
            <CommandShortcut>N S</CommandShortcut>
          </CommandItem>

          <CommandItem
            onSelect={() =>
              handleSelect(() => {
                onSelectTab("payments");
              })
            }
          >
            <CreditCard className="h-4 w-4 text-amber-500" />
            <span>Receive Fee Payment</span>
            <CommandShortcut>N P</CommandShortcut>
          </CommandItem>

          <CommandItem
            onSelect={() =>
              handleSelect(() => {
                onOpenThemeCustomizer();
              })
            }
          >
            <Palette className="h-4 w-4 text-violet-500" />
            <span>Open Appearance & Theme Customizer</span>
            <CommandShortcut>Alt T</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
