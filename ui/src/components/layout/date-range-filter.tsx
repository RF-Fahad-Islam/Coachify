import { useState, useRef, useEffect } from "react";
import { Calendar, ChevronDown, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface DateRangePreset {
  id: string;
  label: string;
  displayValue: string;
}

export const DATE_RANGE_PRESETS: DateRangePreset[] = [
  { id: "today", label: "Today", displayValue: "Apr 21, 2026" },
  { id: "week", label: "This Week", displayValue: "Apr 21, 2026 – Apr 27, 2026" },
  { id: "month", label: "This Month", displayValue: "Apr 1, 2026 – Apr 30, 2026" },
  { id: "last30", label: "Last 30 Days", displayValue: "Mar 28, 2026 – Apr 27, 2026" },
  { id: "term", label: "Academic Term", displayValue: "Jan 1, 2026 – Jun 30, 2026" },
];

export interface DateRangeFilterProps {
  className?: string;
  value?: string;
  onRangeChange?: (presetId: string) => void;
}

export function DateRangeFilter({
  className,
  value,
  onRangeChange,
}: DateRangeFilterProps) {
  const [selectedId, setSelectedId] = useState<string>(value || "week");
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (value && value !== selectedId) {
      setSelectedId(value);
    }
  }, [value, selectedId]);

  const selectedPreset =
    DATE_RANGE_PRESETS.find((p) => p.id === selectedId) || DATE_RANGE_PRESETS[1];

  useEffect(() => {
    if (!open) return;
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
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

  const handleSelect = (preset: DateRangePreset) => {
    setSelectedId(preset.id);
    onRangeChange?.(preset.id);
    setOpen(false);
  };

  return (
    <div className={cn("relative", className)} ref={containerRef}>
      <Button
        variant="outline"
        size="sm"
        data-testid="date-range-filter-btn"
        onClick={() => setOpen((prev) => !prev)}
        aria-label="Filter date range"
        className={cn(
          "hidden lg:inline-flex text-xs gap-2 font-medium text-foreground bg-card hover:bg-muted/80 border-border/80 rounded-xl h-9 px-3.5 shadow-2xs transition-all cursor-pointer",
          open && "ring-2 ring-primary/20 border-primary"
        )}
      >
        <Calendar className="h-3.5 w-3.5 text-primary shrink-0" />
        <span>{selectedPreset.displayValue}</span>
        <ChevronDown
          className={cn(
            "h-3.5 w-3.5 opacity-60 transition-transform",
            open && "rotate-180"
          )}
        />
      </Button>

      {open && (
        <div
          role="menu"
          aria-label="Date range presets"
          className="absolute right-0 top-full mt-2 w-64 rounded-2xl border border-border/80 bg-card text-card-foreground shadow-xl z-50 p-1.5 animate-in fade-in-50 zoom-in-95 duration-150"
        >
          <div className="px-3 py-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground border-b border-border/60 mb-1">
            Date Range Filter
          </div>
          <div className="space-y-0.5">
            {DATE_RANGE_PRESETS.map((preset) => {
              const isSelected = preset.id === selectedId;
              return (
                <button
                  key={preset.id}
                  type="button"
                  data-testid={`date-preset-${preset.id}`}
                  onClick={() => handleSelect(preset)}
                  className={cn(
                    "w-full flex items-center justify-between px-3 py-2 text-xs rounded-xl text-left transition-colors cursor-pointer",
                    isSelected
                      ? "bg-primary-subtle text-primary font-semibold"
                      : "text-foreground hover:bg-muted/70"
                  )}
                >
                  <div>
                    <div className="font-medium leading-none">{preset.label}</div>
                    <div className="text-[11px] text-muted-foreground mt-0.5">
                      {preset.displayValue}
                    </div>
                  </div>
                  {isSelected && (
                    <Check className="h-3.5 w-3.5 text-primary shrink-0 ml-2" />
                  )}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
