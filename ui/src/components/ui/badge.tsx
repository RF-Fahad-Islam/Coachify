import { forwardRef, type HTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const badgeVariants = cva(
  "inline-flex items-center gap-1 font-semibold transition-colors select-none shadow-2xs",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow-xs",
        secondary: "bg-muted text-muted-foreground border border-border/60",
        paid: "bg-success-subtle text-success-text border border-success-border font-semibold",
        pending:
          "bg-warning-subtle text-warning-text border border-warning-border font-semibold",
        overdue:
          "bg-danger-subtle text-danger-text border border-danger-border font-semibold",
        active: "bg-info-subtle text-info-text border border-info-border font-semibold",
        outline: "text-foreground border border-border bg-card/60",
      },
      size: {
        sm: "px-2.5 py-0.5 text-[11px] rounded-full min-h-[20px]",
        md: "px-3 py-1 text-xs rounded-full min-h-[24px]",
        lg: "px-3.5 py-1.5 text-sm rounded-full min-h-[28px]",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  }
);

export interface BadgeProps
  extends HTMLAttributes<HTMLSpanElement>, VariantProps<typeof badgeVariants> {}

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <span
        ref={ref}
        className={cn(badgeVariants({ variant, size, className }))}
        {...props}
      />
    );
  }
);
Badge.displayName = "Badge";
