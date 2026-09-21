import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { ChevronRight, TrendingUp, TrendingDown } from "lucide-react";
import { cn } from "@/lib/utils";

export interface MetricCardProps extends HTMLAttributes<HTMLDivElement> {
  title: string;
  value: string | number;
  subtitle?: string;
  icon?: ReactNode;
  iconBg?: string;
  iconColor?: string;
  trend?: {
    value: string;
    isPositive?: boolean;
    isNeutral?: boolean;
  };
  onClick?: () => void;
}

export const MetricCard = forwardRef<HTMLDivElement, MetricCardProps>(
  (
    {
      className,
      title,
      value,
      subtitle,
      icon,
      iconBg = "bg-primary-subtle",
      iconColor = "text-primary",
      trend,
      onClick,
      ...props
    },
    ref
  ) => {
    return (
      <div
        ref={ref}
        onClick={onClick}
        className={cn(
          "relative overflow-hidden rounded-[var(--radius-lg)] border border-border/80 bg-card p-5 shadow-sm transition-all duration-200 hover:shadow-md hover:border-primary/30 hover:-translate-y-0.5 cursor-pointer",
          className
        )}
        {...props}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            {icon && (
              <div
                className={cn(
                  "flex h-11 w-11 shrink-0 items-center justify-center rounded-[var(--radius-md)]",
                  iconBg,
                  iconColor
                )}
              >
                {icon}
              </div>
            )}
            <div>
              <p className="text-xs font-medium text-muted-foreground">{title}</p>
              <div className="mt-1 flex items-baseline gap-2">
                <span className="text-2xl font-bold tracking-tight text-foreground">
                  {value}
                </span>
                {trend && (
                  <span
                    className={cn(
                      "inline-flex items-center gap-0.5 rounded-full px-2 py-0.5 text-[11px] font-semibold",
                      trend.isNeutral
                        ? "bg-muted text-muted-foreground"
                        : trend.isPositive
                          ? "bg-success-subtle text-success-text"
                          : "bg-danger-subtle text-danger-text"
                    )}
                  >
                    {trend.isPositive ? (
                      <TrendingUp className="h-3 w-3" />
                    ) : (
                      <TrendingDown className="h-3 w-3" />
                    )}
                    {trend.value}
                  </span>
                )}
              </div>
            </div>
          </div>
          {onClick && (
            <ChevronRight className="h-4 w-4 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
          )}
        </div>
        {subtitle && <p className="mt-3 text-xs text-muted-foreground/80">{subtitle}</p>}
      </div>
    );
  }
);
MetricCard.displayName = "MetricCard";
