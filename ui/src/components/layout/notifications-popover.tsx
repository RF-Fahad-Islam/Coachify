import { useState, useRef, useEffect } from "react";
import {
  Bell,
  Check,
  CheckCheck,
  AlertTriangle,
  CalendarCheck,
  ShieldCheck,
  ChevronRight,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { rpc } from "@/gen/rpc";

export interface NotificationItem {
  id: string;
  title: string;
  description: string;
  time: string;
  type: "warning" | "info" | "success";
  targetTab?: string;
  read: boolean;
}

const INITIAL_NOTIFICATIONS: NotificationItem[] = [
  {
    id: "notif-1",
    title: "Overdue Receivables Alert",
    description: "46 students have unpaid fees past 30 days overdue.",
    time: "10m ago",
    type: "warning",
    targetTab: "payments",
    read: false,
  },
  {
    id: "notif-2",
    title: "Batch Attendance Pending",
    description: "HSC Physics Champions (Room 201) roster attendance pending.",
    time: "25m ago",
    type: "info",
    targetTab: "attendance",
    read: false,
  },
  {
    id: "notif-3",
    title: "Nightly Backup Verified",
    description: "Argon2id + AES-256-GCM database snapshot verified healthy.",
    time: "1h ago",
    type: "success",
    targetTab: "settings",
    read: false,
  },
  {
    id: "notif-4",
    title: "SMS Outbox Cleared",
    description: "412 fee receipts and attendance alerts delivered to parents.",
    time: "3h ago",
    type: "info",
    targetTab: "communication",
    read: true,
  },
];

const READ_STORAGE_KEY = "coachify_notifications_read_v1";
const CLEARED_STORAGE_KEY = "coachify_notifications_cleared_v1";

interface NotificationsPopoverProps {
  onNavigateTab?: (tabId: string) => void;
}

export function NotificationsPopover({ onNavigateTab }: NotificationsPopoverProps) {
  const [open, setOpen] = useState(false);
  const [notifications, setNotifications] =
    useState<NotificationItem[]>(INITIAL_NOTIFICATIONS);
  const [filter, setFilter] = useState<"all" | "unread">("all");
  const popoverRef = useRef<HTMLDivElement>(null);

  // Sync with localStorage & core intelligence alerts on mount
  useEffect(() => {
    let mounted = true;
    let readIds: string[] = [];
    let clearedIds: string[] = [];

    try {
      const storedRead = localStorage.getItem(READ_STORAGE_KEY);
      if (storedRead) readIds = JSON.parse(storedRead);
      const storedCleared = localStorage.getItem(CLEARED_STORAGE_KEY);
      if (storedCleared) clearedIds = JSON.parse(storedCleared);
    } catch {
      // Fallback
    }

    const mergeNotifications = (liveAlerts: NotificationItem[]) => {
      if (!mounted) return;
      const combined = [...liveAlerts, ...INITIAL_NOTIFICATIONS].filter(
        (item) => !clearedIds.includes(item.id)
      );

      // De-duplicate by id
      const uniqueMap = new Map<string, NotificationItem>();
      for (const item of combined) {
        if (!uniqueMap.has(item.id)) {
          uniqueMap.set(item.id, {
            ...item,
            read: readIds.includes(item.id) || item.read,
          });
        }
      }

      setNotifications(Array.from(uniqueMap.values()));
    };

    // Apply local storage first
    mergeNotifications([]);

    // Fetch live alerts from core intelligence
    rpc("intelligence.get_today_cockpit", {})
      .then((res) => {
        if (!mounted || !res.ok || !res.data) return;
        const live: NotificationItem[] = [];

        for (const a of res.data.missing_attendance_alerts) {
          live.push({
            id: `alert-missing-att-${a.batch_id}`,
            title: `Missing Attendance: ${a.batch_name}`,
            description: `Class ended at ${a.end_time}. Roster has ${a.enrolled_count} students.`,
            time: "Action required",
            type: "warning",
            targetTab: "attendance",
            read: false,
          });
        }

        for (const t of res.data.absent_teachers) {
          live.push({
            id: `alert-absent-${t.teacher_id}`,
            title: `Faculty Absence: ${t.teacher_name}`,
            description: `Missed class slot for ${t.batch_name} (${t.missed_slot}).`,
            time: "Action required",
            type: "warning",
            targetTab: "attendance",
            read: false,
          });
        }

        if (res.data.overdue_students_count > 0) {
          live.push({
            id: "alert-overdue-receivables",
            title: "Overdue Receivables Alert",
            description: `${res.data.overdue_students_count} students have overdue fees total ৳ ${(res.data.total_overdue_poisha / 100).toLocaleString()}.`,
            time: "Today",
            type: "warning",
            targetTab: "payments",
            read: false,
          });
        }

        if (live.length > 0) {
          mergeNotifications(live);
        }
      })
      .catch(() => {});

    return () => {
      mounted = false;
    };
  }, []);

  const unreadCount = notifications.filter((n) => !n.read).length;

  // Click outside to close
  useEffect(() => {
    if (!open) return;
    function handleClickOutside(event: MouseEvent) {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  const handleMarkAllRead = () => {
    setNotifications((prev) => {
      const updated = prev.map((n) => ({ ...n, read: true }));
      try {
        localStorage.setItem(READ_STORAGE_KEY, JSON.stringify(updated.map((n) => n.id)));
      } catch {
        // Ignore
      }
      return updated;
    });
  };

  const handleClearRead = () => {
    setNotifications((prev) => {
      const clearedIds = prev.filter((n) => n.read).map((n) => n.id);
      const remaining = prev.filter((n) => !n.read);
      try {
        const storedCleared = localStorage.getItem(CLEARED_STORAGE_KEY);
        const existing: string[] = storedCleared ? JSON.parse(storedCleared) : [];
        localStorage.setItem(
          CLEARED_STORAGE_KEY,
          JSON.stringify(Array.from(new Set([...existing, ...clearedIds])))
        );
      } catch {
        // Ignore
      }
      return remaining;
    });
  };

  const handleItemClick = (item: NotificationItem) => {
    // Mark as read
    setNotifications((prev) => {
      const updated = prev.map((n) => (n.id === item.id ? { ...n, read: true } : n));
      try {
        const storedRead = localStorage.getItem(READ_STORAGE_KEY);
        const existing: string[] = storedRead ? JSON.parse(storedRead) : [];
        localStorage.setItem(
          READ_STORAGE_KEY,
          JSON.stringify(Array.from(new Set([...existing, item.id])))
        );
      } catch {
        // Ignore
      }
      return updated;
    });
    if (item.targetTab && onNavigateTab) {
      onNavigateTab(item.targetTab);
      setOpen(false);
    }
  };

  const filteredItems = notifications.filter((n) =>
    filter === "unread" ? !n.read : true
  );

  const getIcon = (type: NotificationItem["type"]) => {
    switch (type) {
      case "warning":
        return <AlertTriangle className="h-4 w-4 text-warning" />;
      case "success":
        return <ShieldCheck className="h-4 w-4 text-success" />;
      case "info":
      default:
        return <CalendarCheck className="h-4 w-4 text-info" />;
    }
  };

  return (
    <div className="relative" ref={popoverRef}>
      <Button
        variant="ghost"
        size="icon"
        onClick={() => setOpen((prev) => !prev)}
        aria-label="Open notifications menu"
        className={cn(
          "relative text-muted-foreground hover:text-foreground hover:bg-muted cursor-pointer h-9 w-9 rounded-xl transition-all duration-150",
          open && "bg-muted text-foreground ring-2 ring-primary/20"
        )}
      >
        <Bell className="h-4 w-4" />
      </Button>

      {unreadCount > 0 && (
        <span
          data-testid="notifications-unread-count"
          className="absolute top-1.5 right-1.5 flex h-4 w-4 items-center justify-center rounded-full bg-primary text-[10px] sm:text-[11px] font-bold text-primary-foreground pointer-events-none shadow-xs"
        >
          {unreadCount}
        </span>
      )}

      {open && (
        <div
          role="dialog"
          aria-label="Notifications panel"
          className="absolute right-0 top-full mt-2 w-80 sm:w-96 rounded-2xl border border-border/80 bg-card text-card-foreground shadow-2xl z-50 overflow-hidden animate-in fade-in-50 zoom-in-95 duration-150"
        >
          {/* Header */}
          <div className="p-4 border-b border-border/70 flex items-center justify-between bg-muted/30">
            <div className="flex items-center gap-2">
              <h4 className="font-bold text-sm text-foreground">Notifications</h4>
              {unreadCount > 0 && (
                <Badge variant="default" size="sm">
                  {unreadCount} new
                </Badge>
              )}
            </div>
            <div className="flex items-center gap-1">
              {unreadCount > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleMarkAllRead}
                  className="h-7 px-2 text-xs font-medium text-muted-foreground hover:text-foreground gap-1"
                >
                  <CheckCheck className="h-3.5 w-3.5" />
                  Mark all read
                </Button>
              )}
            </div>
          </div>

          {/* Sub-filter tabs */}
          <div className="flex border-b border-border/60 bg-muted/20 px-3 py-1.5 gap-2">
            <button
              type="button"
              onClick={() => setFilter("all")}
              className={cn(
                "px-2.5 py-1 text-xs font-semibold rounded-lg transition-colors cursor-pointer",
                filter === "all"
                  ? "bg-card text-foreground shadow-2xs border border-border"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              All ({notifications.length})
            </button>
            <button
              type="button"
              onClick={() => setFilter("unread")}
              className={cn(
                "px-2.5 py-1 text-xs font-semibold rounded-lg transition-colors cursor-pointer",
                filter === "unread"
                  ? "bg-card text-foreground shadow-2xs border border-border"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              Unread ({unreadCount})
            </button>
          </div>

          {/* Notifications List */}
          <div className="max-h-80 overflow-y-auto divide-y divide-border/50">
            {filteredItems.length === 0 ? (
              <div className="py-10 text-center text-muted-foreground text-xs space-y-1">
                <Check className="h-8 w-8 mx-auto text-muted-foreground/40 mb-1" />
                <p className="font-medium text-foreground">All caught up!</p>
                <p>No {filter === "unread" ? "unread" : ""} notifications right now.</p>
              </div>
            ) : (
              filteredItems.map((item) => (
                <div
                  key={item.id}
                  onClick={() => handleItemClick(item)}
                  className={cn(
                    "p-3.5 flex items-start gap-3 transition-colors cursor-pointer group hover:bg-muted/40",
                    !item.read && "bg-primary-subtle/30"
                  )}
                >
                  <div className="mt-0.5 shrink-0 rounded-lg p-1.5 bg-card border border-border shadow-2xs">
                    {getIcon(item.type)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-1 mb-0.5">
                      <span
                        className={cn(
                          "text-xs font-semibold truncate",
                          item.read ? "text-foreground/80" : "text-foreground font-bold"
                        )}
                      >
                        {item.title}
                      </span>
                      <span className="text-[11px] text-muted-foreground shrink-0 font-medium">
                        {item.time}
                      </span>
                    </div>
                    <p className="text-xs text-muted-foreground leading-relaxed line-clamp-2">
                      {item.description}
                    </p>
                  </div>
                  <ChevronRight className="h-4 w-4 text-muted-foreground/60 transition-transform group-hover:translate-x-0.5 shrink-0 self-center" />
                </div>
              ))
            )}
          </div>

          {/* Footer */}
          {notifications.some((n) => n.read) && (
            <div className="p-2 border-t border-border/70 bg-muted/20 flex justify-end">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleClearRead}
                className="h-7 px-2 text-xs text-muted-foreground hover:text-foreground gap-1.5"
              >
                <Trash2 className="h-3 w-3" />
                Clear Read
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
