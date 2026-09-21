import { useState, useEffect } from "react";
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
  GraduationCap,
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
import { rpc, type StudentSearchHit } from "@/gen/rpc";

export interface QuickSearchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectTab: (tabId: string) => void;
  onOpenThemeCustomizer: () => void;
  onSelectStudent?: (studentId: string) => void;
}

export function QuickSearchDialog({
  open,
  onOpenChange,
  onSelectTab,
  onOpenThemeCustomizer,
  onSelectStudent,
}: QuickSearchDialogProps) {
  const [query, setQuery] = useState("");
  const [studentMatches, setStudentMatches] = useState<StudentSearchHit[]>([]);

  useEffect(() => {
    if (!open || query.trim().length < 2) {
      setStudentMatches([]);
      return;
    }

    const timer = setTimeout(async () => {
      try {
        const res = await rpc("students.search", { query: query.trim(), limit: 5 });
        if (res.ok && res.data) {
          setStudentMatches(res.data.matches || []);
        }
      } catch (e) {
        console.error("Failed to query students in quick search", e);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [query, open]);

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
        {filteredNav.length === 0 && studentMatches.length === 0 && (
          <CommandEmpty>No matching commands or students found.</CommandEmpty>
        )}

        {studentMatches.length > 0 && (
          <CommandGroup heading="Students">
            {studentMatches.map((student) => (
              <CommandItem
                key={student.id}
                onSelect={() =>
                  handleSelect(() => {
                    if (onSelectStudent) {
                      onSelectStudent(student.id);
                    } else {
                      onSelectTab("students");
                    }
                  })
                }
              >
                <GraduationCap className="h-4 w-4 text-primary shrink-0" />
                <div className="flex flex-col flex-1 min-w-0 text-left">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-foreground">{student.name}</span>
                    <span className="font-mono text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
                      {student.student_code}
                    </span>
                  </div>
                  <span className="text-[11px] text-muted-foreground truncate">
                    {student.phone_norm}{" "}
                    {student.guardian_names
                      ? `• Guardian: ${student.guardian_names}`
                      : ""}
                  </span>
                </div>
              </CommandItem>
            ))}
          </CommandGroup>
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
