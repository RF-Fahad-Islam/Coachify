import { forwardRef, type ButtonHTMLAttributes } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

export const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 font-medium transition-colors cursor-pointer select-none disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-primary-foreground hover:bg-primary-hover shadow-xs active:scale-[0.99]",
        secondary: "bg-muted text-foreground hover:bg-muted/80 active:scale-[0.99]",
        outline:
          "border border-border bg-card hover:bg-muted text-foreground active:scale-[0.99]",
        ghost: "text-foreground hover:bg-muted/70 active:scale-[0.99]",
        destructive:
          "bg-danger text-danger-foreground hover:bg-danger/90 shadow-xs active:scale-[0.99]",
        subtle:
          "bg-primary-subtle text-primary hover:bg-primary-subtle-hover border border-primary-border",
      },
      size: {
        sm: "h-8 px-3 text-xs rounded-[var(--radius-sm)]",
        md: "h-[var(--btn-height)] px-[var(--btn-px)] py-[var(--btn-py)] text-sm rounded-[var(--radius)]",
        lg: "h-11 px-6 text-base rounded-[var(--radius-md)]",
        icon: "h-[var(--btn-height)] w-[var(--btn-height)] p-0 rounded-[var(--radius)]",
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
