import { useState, useEffect, useCallback } from "react";
import { rpc, type CoachingDayTimelineResponse, type TimelineEvent } from "@/gen/rpc";
import { formatBusinessDate } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Clock,
  CreditCard,
  UserPlus,
  CalendarCheck,
  GraduationCap,
  MessageSquare,
  Settings,
  FileText,
  Filter,
  RefreshCw,
  Loader2,
  ChevronLeft,
  ChevronRight,
  User,
} from "lucide-react";

export type TimelineCategoryFilter =
  "all" | "finance" | "admissions" | "attendance" | "exams" | "messages" | "settings";

const CATEGORY_TABS: {
  id: TimelineCategoryFilter;
  label: string;
  icon: typeof Filter;
}[] = [
  { id: "all", label: "All Activity", icon: Filter },
  { id: "finance", label: "Finance & Cash", icon: CreditCard },
  { id: "admissions", label: "Admissions", icon: UserPlus },
  { id: "attendance", label: "Attendance", icon: CalendarCheck },
  { id: "exams", label: "Exams & Marks", icon: GraduationCap },
  { id: "messages", label: "Comms & SMS", icon: MessageSquare },
  { id: "settings", label: "Administrative", icon: Settings },
];

export function CoachingDayView() {
  const [selectedDate, setSelectedDate] = useState<string>(() => {
    return new Date().toISOString().slice(0, 10);
  });
  const [activeCategory, setActiveCategory] = useState<TimelineCategoryFilter>("all");
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [timelineData, setTimelineData] = useState<CoachingDayTimelineResponse | null>(
    null
  );
  const [events, setEvents] = useState<TimelineEvent[]>([]);

  const loadTimeline = useCallback(
    async (date: string, category: TimelineCategoryFilter) => {
      setLoading(true);
      setError(null);
      try {
        const res = await rpc("intelligence.get_coaching_day_timeline", {
          date,
          filter_category: category === "all" ? null : category,
          limit: 50,
        });

        if (res.ok && res.data) {
          setTimelineData(res.data);
          setEvents(res.data.events);
        } else {
          setError(res.error?.message || "Failed to load coaching day timeline");
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    },
    []
  );

  useEffect(() => {
    void loadTimeline(selectedDate, activeCategory);
  }, [selectedDate, activeCategory, loadTimeline]);

  const handleLoadMore = async () => {
    if (!timelineData?.next_cursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const res = await rpc("intelligence.get_coaching_day_timeline", {
        date: selectedDate,
        filter_category: activeCategory === "all" ? null : activeCategory,
        limit: 50,
        before_time: timelineData.next_cursor.before_time,
        before_id: timelineData.next_cursor.before_id,
      });

      if (res.ok && res.data) {
        const nextData = res.data;
        setTimelineData(nextData);
        setEvents((prev) => [...prev, ...nextData.events]);
      }
    } catch (err: unknown) {
      console.error("Failed to load more timeline events", err);
    } finally {
      setLoadingMore(false);
    }
  };

  const handleShiftDate = (days: number) => {
    const current = new Date(selectedDate);
    current.setDate(current.getDate() + days);
    setSelectedDate(current.toISOString().slice(0, 10));
  };

  const getCategoryIcon = (category: string) => {
    switch (category.toLowerCase()) {
      case "finance":
      case "payment":
        return <CreditCard className="h-3.5 w-3.5 text-emerald-600" />;
      case "admission":
      case "admissions":
        return <UserPlus className="h-3.5 w-3.5 text-blue-600" />;
      case "attendance":
        return <CalendarCheck className="h-3.5 w-3.5 text-purple-600" />;
      case "exam":
      case "exams":
        return <GraduationCap className="h-3.5 w-3.5 text-amber-600" />;
      case "message":
      case "messages":
      case "comms":
        return <MessageSquare className="h-3.5 w-3.5 text-indigo-600" />;
      default:
        return <FileText className="h-3.5 w-3.5 text-muted-foreground" />;
    }
  };

  const getCategoryBadgeClass = (category: string) => {
    switch (category.toLowerCase()) {
      case "finance":
      case "payment":
        return "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/20";
      case "admission":
      case "admissions":
        return "bg-blue-500/10 text-blue-700 dark:text-blue-300 border-blue-500/20";
      case "attendance":
        return "bg-purple-500/10 text-purple-700 dark:text-purple-300 border-purple-500/20";
      case "exam":
      case "exams":
        return "bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/20";
      case "message":
      case "messages":
      case "comms":
        return "bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border-indigo-500/20";
      default:
        return "bg-muted text-muted-foreground border-border";
    }
  };

  const isToday = selectedDate === new Date().toISOString().slice(0, 10);

  return (
    <div className="space-y-6">
      {/* Top Date & Controls Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-bold tracking-tight text-foreground">
              Coaching Day Timeline
            </h2>
            <Badge variant="outline" size="sm" className="font-mono">
              {events.length} activities
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-0.5">
            Immutable single-day activity ledger: counter collections, admissions, roll
            calls, exam publishing, and administrative actions.
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <div className="flex items-center bg-card border border-border rounded-[var(--radius-md)] p-1 shadow-2xs">
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={() => handleShiftDate(-1)}
              title="Previous Day"
              aria-label="Previous day"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <input
              type="date"
              aria-label="Select Target Date"
              value={selectedDate}
              onChange={(e) => e.target.value && setSelectedDate(e.target.value)}
              className="px-2 py-0.5 text-xs font-semibold bg-transparent border-0 focus:outline-hidden text-foreground cursor-pointer"
            />
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={() => handleShiftDate(1)}
              title="Next Day"
              aria-label="Next day"
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>

          {!isToday && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedDate(new Date().toISOString().slice(0, 10))}
              className="text-xs h-8 font-medium"
            >
              Today
            </Button>
          )}

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadTimeline(selectedDate, activeCategory)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh timeline data"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {/* Category Filter Pills */}
      <div className="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-thin">
        {CATEGORY_TABS.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeCategory === tab.id;
          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveCategory(tab.id)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-semibold whitespace-nowrap transition-colors cursor-pointer border ${
                isActive
                  ? "bg-primary text-primary-foreground border-primary shadow-2xs"
                  : "bg-card text-muted-foreground border-border hover:text-foreground hover:bg-muted/60"
              }`}
            >
              <Icon className="h-3.5 w-3.5" />
              <span>{tab.label}</span>
            </button>
          );
        })}
      </div>

      {/* Error state */}
      {error && (
        <div className="p-4 rounded-[var(--radius-lg)] bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center justify-between">
          <span>{error}</span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadTimeline(selectedDate, activeCategory)}
            className="h-7 text-xs border-destructive/30"
          >
            Retry
          </Button>
        </div>
      )}

      {/* Loading state */}
      {loading ? (
        <div className="py-20 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Loading timeline events for {formatBusinessDate(selectedDate)}...
          </span>
        </div>
      ) : events.length === 0 ? (
        /* Empty state */
        <Card className="border-dashed border-2 border-border">
          <CardContent className="py-16 text-center space-y-3">
            <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center mx-auto text-muted-foreground">
              <Clock className="h-6 w-6" />
            </div>
            <h3 className="text-sm font-bold text-foreground">
              No activity recorded for {formatBusinessDate(selectedDate)}
            </h3>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              No payments, attendances, admissions, or administrative actions occurred
              under the selected filter on this date.
            </p>
          </CardContent>
        </Card>
      ) : (
        /* Timeline Feed */
        <div className="space-y-4">
          <div className="relative pl-6 sm:pl-8 before:absolute before:left-3 sm:before:left-4 before:top-3 before:bottom-3 before:w-0.5 before:bg-border">
            <div className="space-y-4">
              {events.map((event, idx) => {
                const timeStr = event.event_time.includes("T")
                  ? event.event_time.split("T")[1]?.slice(0, 5) ||
                    event.event_time.slice(11, 16)
                  : event.event_time.slice(11, 16);

                return (
                  <div
                    key={event.id || `${event.event_time}-${idx}`}
                    className="relative group"
                  >
                    {/* Node Dot */}
                    <div className="absolute -left-[27px] sm:-left-[31px] top-1.5 h-6 w-6 rounded-full bg-card border-2 border-primary flex items-center justify-center shadow-2xs group-hover:scale-110 transition-transform">
                      {getCategoryIcon(event.category)}
                    </div>

                    {/* Content Card */}
                    <div className="p-4 rounded-[var(--radius-lg)] border border-border/80 bg-card hover:border-border hover:shadow-2xs transition-all space-y-2">
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="font-bold text-sm text-foreground">
                            {event.title}
                          </span>
                          <span
                            className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border ${getCategoryBadgeClass(
                              event.category
                            )}`}
                          >
                            {event.category}
                          </span>
                          <span className="text-[11px] font-mono text-muted-foreground">
                            {event.event_type}
                          </span>
                        </div>

                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1 font-mono font-medium">
                            <Clock className="h-3 w-3" />
                            {timeStr || event.event_time.slice(0, 16)}
                          </span>
                        </div>
                      </div>

                      <p className="text-xs text-foreground/90 leading-relaxed">
                        {event.description}
                      </p>

                      <div className="flex items-center justify-between pt-1 border-t border-border/50 text-[11px] text-muted-foreground">
                        <div className="flex items-center gap-1.5">
                          <User className="h-3 w-3 text-muted-foreground/70" />
                          <span>
                            {event.actor_name ? (
                              <>
                                Recorded by{" "}
                                <strong className="text-foreground">
                                  {event.actor_name}
                                </strong>
                                {event.actor_username && ` (${event.actor_username})`}
                              </>
                            ) : (
                              "Automated system activity"
                            )}
                          </span>
                        </div>

                        {event.metadata_json && (
                          <div className="text-[10px] font-mono text-muted-foreground bg-muted/60 px-2 py-0.5 rounded">
                            Details attached
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Load More Button */}
          {timelineData?.has_more && (
            <div className="text-center pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleLoadMore}
                disabled={loadingMore}
                className="text-xs font-semibold gap-1.5"
              >
                {loadingMore ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <Clock className="h-3.5 w-3.5" />
                )}
                Load Older Activity
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
