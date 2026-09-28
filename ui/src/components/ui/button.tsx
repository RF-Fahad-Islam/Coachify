import { forwardRef, type ButtonHTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 font-medium transition-all duration-150 cursor-pointer select-none disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 active:scale-[0.98]",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-primary-foreground hover:bg-primary-hover shadow-xs hover:shadow-sm font-semibold",
        secondary: "bg-muted text-foreground hover:bg-muted/80 font-medium",
        outline:
          "border border-border bg-card hover:bg-muted text-foreground shadow-2xs font-medium",
        ghost: "text-foreground hover:bg-muted/70 font-medium",
        destructive:
          "bg-danger text-danger-foreground hover:bg-danger/90 shadow-xs font-semibold",
        subtle:
          "bg-primary-subtle text-primary hover:bg-primary-subtle-hover border border-primary-border font-semibold",
      },
      size: {
        sm: "h-8 px-3.5 text-xs rounded-[var(--radius-sm)] font-medium",
        md: "h-[var(--btn-height)] px-[var(--btn-px)] py-[var(--btn-py)] text-sm rounded-[var(--radius-md)] font-medium shadow-xs",
        lg: "h-11 sm:h-12 px-6 text-base rounded-[var(--radius-lg)] font-semibold shadow-xs",
        icon: "h-9 w-9 sm:h-[var(--btn-height)] sm:w-[var(--btn-height)] p-0 rounded-[var(--radius-md)]",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "md",
    },
  }
);

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(buttonVariants({ variant, size, className }))}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";
