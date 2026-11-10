import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?:
    | "primary"
    | "primary-soft"
    | "secondary"
    | "outline"
    | "warning"
    | "danger"
    | "danger-ghost"
    | "ghost";
  size?: "sm" | "md" | "lg";
}

export const Button: React.FC<ButtonProps> = ({
  children,
  className = "",
  variant = "primary",
  size = "md",
  ...props
}) => {
  const baseClasses =
    "inline-flex items-center justify-center gap-1.5 text-center font-medium rounded-[10px] border focus:outline-none transition-[background-color,border-color,color,transform] duration-150 active:scale-[0.97] disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100 cursor-pointer";

  const variantClasses = {
    primary:
      "text-white bg-background-ui border-background-ui hover:bg-background-ui/90 hover:border-background-ui/90 focus-visible:ring-2 focus-visible:ring-background-ui/40",
    "primary-soft":
      "text-text bg-logo-primary/20 border-transparent hover:bg-logo-primary/20 focus:ring-1 focus:ring-logo-primary",
    secondary:
      "text-text bg-wash border-transparent hover:bg-line focus:outline-none",
    // For screens where secondary's grey fill reads as disabled (onboarding)
    outline:
      "text-text bg-background border-line-strong hover:border-plum focus-visible:ring-1 focus-visible:ring-logo-primary",
    // Secondary's neutral resting look, but hover/focus use the semantic
    // --color-warning token (theme.css) instead of the pink accent — for
    // buttons sitting on warning surfaces like SecureInputWarning
    warning:
      "text-text bg-wash border-line hover:bg-warning-wash hover:border-warning focus:ring-1 focus:ring-warning",
    danger:
      "text-background bg-error border-error hover:bg-error/80 hover:border-error/80 focus:ring-1 focus:ring-error",
    "danger-ghost":
      "text-error border-transparent hover:bg-error-wash focus:bg-error-wash",
    ghost:
      "text-current border-transparent hover:bg-wash focus-visible:bg-line",
  };

  const sizeClasses = {
    sm: "min-h-[28px] px-2.5 text-xs",
    md: "min-h-[32px] px-4 text-sm",
    lg: "min-h-[40px] px-5 text-base",
  };

  return (
    <button
      className={`${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};
