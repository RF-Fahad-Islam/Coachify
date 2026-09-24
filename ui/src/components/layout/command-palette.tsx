import { useState, useEffect, useCallback, useMemo } from "react";
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
  Receipt,
  UserCheck,
  DollarSign,
  Lock,
  Database,
  BookOpen,
  Keyboard,
  Sparkles,
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
import { Badge } from "@/components/ui/badge";
import { rpc, type GlobalSearchResponse } from "@/gen/rpc";
import { useAuth } from "@/contexts/auth-context";

export interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectTab: (tabId: string) => void;
  onOpenThemeCustomizer: () => void;
  onOpenAdmitStudent?: () => void;
  onOpenPaymentCounter?: () => void;
  onSelectStudent?: (studentId: string) => void;
  onSelectReceipt?: (receiptNumber: string) => void;
  onOpenCopilot?: () => void;
}

export function CommandPalette({
  open,
  onOpenChange,
  onSelectTab,
  onOpenThemeCustomizer,
  onOpenAdmitStudent,
  onOpenPaymentCounter,
  onSelectStudent,
  onSelectReceipt,
  onOpenCopilot,
}: CommandPaletteProps) {
  const { hasPermission } = useAuth();
  const [query, setQuery] = useState("");
  const [showCheatsheet, setShowCheatsheet] = useState(false);
  const [searchResults, setSearchResults] = useState<GlobalSearchResponse | null>(null);
  const [isSearching, setIsSearching] = useState(false);

  // Fast live global search query (AC-1, AC-5: 150ms debounce)
  useEffect(() => {
    if (!open || query.trim().length < 2) {
      setSearchResults(null);
      return;
    }

    const timer = setTimeout(async () => {
      setIsSearching(true);
      try {
        const res = await rpc("search.global", { query: query.trim(), limit: 5 });
        if (res.ok && res.data) {
          setSearchResults(res.data);
        }
      } catch (err) {
        console.error("Global search failed in CommandPalette", err);
      } finally {
        setIsSearching(false);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [query, open]);

  const handleExecute = useCallback(
    (action: () => void) => {
      action();
      onOpenChange(false);
      setQuery("");
    },
    [onOpenChange]
  );

  // Global Chorded Shortcuts Listener (Spec 0030 AC-3)
  useEffect(() => {
    let chordPrefix: string | null = null;
    let chordTimer: NodeJS.Timeout | null = null;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Guard: do not trigger when typing inside an input or editable field
      const target = e.target as HTMLElement | null;
      const isInput =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);

      // Ctrl+K / Cmd+K toggle (works even from inputs)
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onOpenChange(!open);
        return;
      }

      // Alt+T Theme Customizer
      if (e.altKey && e.key.toLowerCase() === "t") {
        e.preventDefault();
        onOpenThemeCustomizer();
        return;
      }

      if (isInput || open) return;

      const key = e.key.toUpperCase();

      if (key === "?") {
        e.preventDefault();
        onOpenChange(true);
        setShowCheatsheet(true);
        return;
      }

      if (chordPrefix) {
        const chord = `${chordPrefix} ${key}`;
        chordPrefix = null;
        if (chordTimer) clearTimeout(chordTimer);

        switch (chord) {
          case "G D":
            e.preventDefault();
            onSelectTab("dashboard");
            break;
          case "G S":
            e.preventDefault();
            onSelectTab("students");
            break;
          case "G A":
            e.preventDefault();
            onSelectTab("attendance");
            break;
          case "G P":
          case "G F":
            e.preventDefault();
            onSelectTab("payments");
            break;
          case "G E":
            e.preventDefault();
            onSelectTab("exams");
            break;
          case "G R":
            e.preventDefault();
            onSelectTab("reports");
            break;
          case "G T":
            e.preventDefault();
            onSelectTab("settings");
            break;
          case "N S":
            e.preventDefault();
            if (onOpenAdmitStudent) {
              onOpenAdmitStudent();
            } else {
              onSelectTab("students");
            }
            break;
          case "N P":
            e.preventDefault();
            if (onOpenPaymentCounter) {
              onOpenPaymentCounter();
            } else {
              onSelectTab("payments");
            }
            break;
          default:
            break;
        }
      } else if (key === "G" || key === "N") {
        chordPrefix = key;
        chordTimer = setTimeout(() => {
          chordPrefix = null;
        }, 1200);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      if (chordTimer) clearTimeout(chordTimer);
    };
  }, [
    open,
    onOpenChange,
    onSelectTab,
    onOpenThemeCustomizer,
    onOpenAdmitStudent,
    onOpenPaymentCounter,
  ]);

  // Quick action definitions with aliases and permission checks (AC-4, AC-5)
  const quickActions = useMemo(
    () => [
      {
        id: "admit-student",
        label: "Admit New Student",
        shortcut: "N S",
        aliases: ["new student", "enroll", "vorti", "admit", "admission"],
        icon: Plus,
        permission: "students.manage",
        action: () =>
          onOpenAdmitStudent ? onOpenAdmitStudent() : onSelectTab("students"),
      },
      {
        id: "receive-payment",
        label: "Receive Fee Payment",
        shortcut: "N P",
        aliases: [
          "pay",
          "payment",
          "fee",
          "cashier",
          "money",
          "taka",
          "counter",
          "collect",
        ],
        icon: DollarSign,
        permission: "payments.collect",
        action: () =>
          onOpenPaymentCounter ? onOpenPaymentCounter() : onSelectTab("payments"),
      },
      {
        id: "mark-attendance",
        label: "Mark Batch Attendance",
        shortcut: "G A",
        aliases: ["attendance", "hajira", "presence", "roll call"],
        icon: CalendarCheck,
        permission: "attendance.mark",
        action: () => onSelectTab("attendance"),
      },
      {
        id: "dues-directory",
        label: "View Dues Directory & Aging",
        shortcut: "G F",
        aliases: ["due", "baki", "overdue", "receivables"],
        icon: CreditCard,
        permission: "reports.financial",
        action: () => onSelectTab("reports"),
      },
      {
        id: "daily-cash-close",
        label: "Daily Cash Drawer Close",
        shortcut: "G P",
        aliases: ["cash close", "drawer", "reconcile", "closing"],
        icon: Lock,
        permission: "payments.view",
        action: () => onSelectTab("payments"),
      },
      {
        id: "create-exam",
        label: "Create New Exam",
        shortcut: "G E",
        aliases: ["exam", "test", "quiz", "marks", "parikkha"],
        icon: FileText,
        permission: "exam.manage",
        action: () => onSelectTab("exams"),
      },
      {
        id: "database-backup",
        label: "Trigger Database Backup",
        shortcut: "G T",
        aliases: ["backup", "snapshot", "save database"],
        icon: Database,
        permission: "backup.manage",
        action: () => onSelectTab("settings"),
      },
      {
        id: "customize-theme",
        label: "Customize Theme & Appearance",
        shortcut: "Alt T",
        aliases: ["theme", "dark mode", "light mode", "color", "appearance"],
        icon: Palette,
        permission: null,
        action: onOpenThemeCustomizer,
      },
      {
        id: "ai-copilot",
        label: "Open Coachify AI Copilot",
        shortcut: "Ctrl J",
        aliases: [
          "ai",
          "copilot",
          "chat",
          "assistant",
          "bot",
          "intelligence",
          "help",
          "ask",
        ],
        icon: Sparkles,
        permission: null,
        action: () => {
          if (onOpenCopilot) {
            onOpenCopilot();
          }
        },
      },
    ],
    [
      onOpenAdmitStudent,
      onOpenPaymentCounter,
      onSelectTab,
      onOpenThemeCustomizer,
      onOpenCopilot,
    ]
  );

  // Navigation module definitions (AC-2)
  const navModules = useMemo(
    () => [
      {
        id: "dashboard",
        label: "Dashboard",
        shortcut: "G D",
        icon: LayoutDashboard,
        permission: null,
      },
      {
        id: "students",
        label: "Students Directory",
        shortcut: "G S",
        icon: Users,
        permission: "students.view",
      },
      {
        id: "attendance",
        label: "Batch Attendance",
        shortcut: "G A",
        icon: CalendarCheck,
        permission: "attendance.mark",
      },
      {
        id: "payments",
        label: "Finance & Fee Ledger",
        shortcut: "G F",
        icon: CreditCard,
        permission: "payments.view",
      },
      {
        id: "exams",
        label: "Exams & Mark Entry",
        shortcut: "G E",
        icon: FileText,
        permission: "exam.view",
      },
      {
        id: "reports",
        label: "Reports Center & Exports",
        shortcut: "G R",
        icon: BarChart3,
        permission: "reports.view",
      },
      {
        id: "settings",
        label: "System & Institute Settings",
        shortcut: "G T",
        icon: Settings,
        permission: "system.settings",
      },
    ],
    []
  );

  // Filter actions based on search query, aliases, and role permissions
  const qLower = query.toLowerCase().trim();

  const filteredActions = useMemo(() => {
    return quickActions.filter((item) => {
      if (item.permission && !hasPermission(item.permission)) return false;
      if (!qLower) return true;
      const matchesLabel = item.label.toLowerCase().includes(qLower);
      const matchesAlias = item.aliases.some((a) => a.toLowerCase().includes(qLower));
      return matchesLabel || matchesAlias;
    });
  }, [quickActions, qLower, hasPermission]);

  const filteredNav = useMemo(() => {
    return navModules.filter((item) => {
      if (item.permission && !hasPermission(item.permission)) return false;
      if (!qLower) return true;
      return item.label.toLowerCase().includes(qLower);
    });
  }, [navModules, qLower, hasPermission]);

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput
        placeholder="Type a command or search students, batches, receipts... (e.g. Pay, Admit, 01711)"
        value={query}
        onValueChange={setQuery}
      />
      <CommandList>
        {isSearching && (
          <div className="py-2 px-3 text-2xs text-muted-foreground animate-pulse">
            Searching database...
          </div>
        )}

        {filteredActions.length === 0 &&
          filteredNav.length === 0 &&
          (!searchResults || searchResults.total_hits === 0) &&
          !isSearching && (
            <CommandEmpty>
              No matching commands, students, or receipts found.
            </CommandEmpty>
          )}

        {/* Live Search Hits: Students */}
        {searchResults && searchResults.students.length > 0 && (
          <CommandGroup heading="Students & Guardians">
            {searchResults.students.map((student) => (
              <CommandItem
                key={student.id}
                onSelect={() =>
                  handleExecute(() => {
                    if (onSelectStudent) {
                      onSelectStudent(student.id);
                    } else {
                      onSelectTab("students");
                    }
                  })
                }
              >
                <GraduationCap className="mr-2 h-4 w-4 text-primary shrink-0" />
                <div className="flex flex-col flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-xs truncate">{student.name}</span>
                    {student.name_bn && (
                      <span className="text-2xs text-muted-foreground font-bangla truncate">
                        {student.name_bn}
                      </span>
                    )}
                    <Badge variant="outline" className="text-3xs px-1 py-0 h-4">
                      {student.student_code}
                    </Badge>
                  </div>
                  <div className="flex items-center gap-2 text-2xs text-muted-foreground">
                    {student.batch_name && <span>{student.batch_name}</span>}
                    {student.matched_guardian_badge && (
                      <span className="text-primary font-medium">
                        • {student.matched_guardian_badge}
                      </span>
                    )}
                  </div>
                </div>
                <CommandShortcut>Student</CommandShortcut>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Live Search Hits: Payment Receipts */}
        {searchResults && searchResults.receipts.length > 0 && (
          <CommandGroup heading="Receipts & Payments">
            {searchResults.receipts.map((rcp) => (
              <CommandItem
                key={rcp.payment_id}
                onSelect={() =>
                  handleExecute(() => {
                    if (onSelectReceipt) {
                      onSelectReceipt(rcp.receipt_number);
                    } else {
                      onSelectTab("payments");
                    }
                  })
                }
              >
                <Receipt className="mr-2 h-4 w-4 text-emerald-600 shrink-0" />
                <div className="flex flex-col flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-xs text-foreground font-mono">
                      {rcp.receipt_number}
                    </span>
                    <span className="text-2xs text-muted-foreground">
                      ৳{rcp.amount_formatted} ({rcp.method.toUpperCase()})
                    </span>
                  </div>
                  <span className="text-2xs text-muted-foreground truncate">
                    {rcp.student_name} ({rcp.student_code}) •{" "}
                    {rcp.paid_at.substring(0, 10)}
                  </span>
                </div>
                <CommandShortcut>Receipt</CommandShortcut>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Live Search Hits: Batches */}
        {searchResults && searchResults.batches.length > 0 && (
          <CommandGroup heading="Batches & Classes">
            {searchResults.batches.map((batch) => (
              <CommandItem
                key={batch.id}
                onSelect={() => handleExecute(() => onSelectTab("attendance"))}
              >
                <BookOpen className="mr-2 h-4 w-4 text-indigo-500 shrink-0" />
                <div className="flex flex-col flex-1 min-w-0">
                  <span className="font-medium text-xs">{batch.name}</span>
                  <span className="text-2xs text-muted-foreground">
                    {batch.code} • {batch.program_name} • {batch.student_count} enrolled
                  </span>
                </div>
                <CommandShortcut>Batch</CommandShortcut>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Live Search Hits: Staff */}
        {searchResults && searchResults.staff.length > 0 && (
          <CommandGroup heading="Faculty & Staff">
            {searchResults.staff.map((stf) => (
              <CommandItem
                key={stf.id}
                onSelect={() => handleExecute(() => onSelectTab("settings"))}
              >
                <UserCheck className="mr-2 h-4 w-4 text-teal-600 shrink-0" />
                <div className="flex flex-col flex-1 min-w-0">
                  <span className="font-medium text-xs">{stf.name}</span>
                  <span className="text-2xs text-muted-foreground">
                    {stf.staff_code} {stf.designation ? `• ${stf.designation}` : ""}
                  </span>
                </div>
                <CommandShortcut>Staff</CommandShortcut>
              </CommandItem>
            ))}
          </CommandGroup>
        )}

        {/* Quick Actions Group */}
        {filteredActions.length > 0 && (
          <CommandGroup heading="Quick Actions">
            {filteredActions.map((action) => {
              const Icon = action.icon;
              return (
                <CommandItem
                  key={action.id}
                  onSelect={() => handleExecute(action.action)}
                >
                  <Icon className="mr-2 h-4 w-4 text-primary shrink-0" />
                  <span className="flex-1 text-xs">{action.label}</span>
                  {action.shortcut && (
                    <CommandShortcut>{action.shortcut}</CommandShortcut>
                  )}
                </CommandItem>
              );
            })}
          </CommandGroup>
        )}

        {/* Navigation Group */}
        {filteredNav.length > 0 && (
          <CommandGroup heading="Navigation">
            {filteredNav.map((item) => {
              const Icon = item.icon;
              return (
                <CommandItem
                  key={item.id}
                  onSelect={() => handleExecute(() => onSelectTab(item.id))}
                >
                  <Icon className="mr-2 h-4 w-4 text-muted-foreground shrink-0" />
                  <span className="flex-1 text-xs">{item.label}</span>
                  <CommandShortcut>{item.shortcut}</CommandShortcut>
                </CommandItem>
              );
            })}
          </CommandGroup>
        )}

        {/* Keyboard Shortcuts Sheet (Triggered with ? or in menu) */}
        {(!query || showCheatsheet) && (
          <CommandGroup heading="Shortcuts Cheat Sheet">
            <CommandItem onSelect={() => setShowCheatsheet(!showCheatsheet)}>
              <Keyboard className="mr-2 h-4 w-4 text-muted-foreground shrink-0" />
              <span className="flex-1 text-xs text-muted-foreground">
                Press{" "}
                <kbd className="px-1 py-0.5 rounded bg-muted font-mono text-3xs">?</kbd>{" "}
                anywhere to show full keyboard cheat sheet
              </span>
            </CommandItem>
          </CommandGroup>
        )}
      </CommandList>
    </CommandDialog>
  );
}
