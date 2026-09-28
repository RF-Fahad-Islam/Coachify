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
          "relative overflow-hidden rounded-[var(--radius-xl)] sm:rounded-[var(--radius-2xl)] border border-border/80 bg-card p-5 sm:p-6 shadow-sm transition-all duration-200 hover:shadow-md hover:border-primary/30 hover:-translate-y-0.5 cursor-pointer group",
          className
        )}
        {...props}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3.5">
            {icon && (
              <div
                className={cn(
                  "flex h-12 w-12 shrink-0 items-center justify-center rounded-[var(--radius-lg)] shadow-2xs transition-transform duration-200 group-hover:scale-105",
                  iconBg,
                  iconColor
                )}
              >
                {icon}
              </div>
            )}
            <div>
              <p className="text-xs sm:text-sm font-semibold text-muted-foreground tracking-normal">
                {title}
              </p>
              <div className="mt-1 flex items-baseline gap-2.5 flex-wrap">
                <span className="text-2xl sm:text-3xl font-extrabold tracking-tight text-foreground">
                  {value}
                </span>
                {trend && (
                  <span
                    className={cn(
                      "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold",
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
            <ChevronRight className="h-4 w-4 text-muted-foreground transition-transform group-hover:translate-x-1" />
          )}
        </div>
        {subtitle && (
          <p className="mt-3 text-xs sm:text-sm text-muted-foreground/90 leading-relaxed">
            {subtitle}
          </p>
        )}
      </div>
    );
  }
);
MetricCard.displayName = "MetricCard";
