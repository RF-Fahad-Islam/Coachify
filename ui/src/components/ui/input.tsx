import { forwardRef, type InputHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, error, ...props }, ref) => {
    return (
      <input
        type={type}
        ref={ref}
        className={cn(
          "flex h-[var(--input-height)] w-full rounded-[var(--radius-md)] border border-input bg-card px-3.5 py-2 text-sm text-foreground shadow-2xs transition-all file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40 focus-visible:border-primary focus-visible:ring-offset-1 disabled:cursor-not-allowed disabled:opacity-50",
          error && "border-danger focus-visible:ring-danger",
          className
        )}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";
