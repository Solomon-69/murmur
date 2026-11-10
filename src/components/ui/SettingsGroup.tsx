import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

/** A small section heading over one soft, rounded card of 44px rows. */
export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  return (
    <section className="space-y-2">
      {title && (
        <div className="px-1">
          <h2 className="text-[13px] font-semibold text-ink-2">{title}</h2>
          {description && (
            <p className="mt-0.5 text-xs text-ink-3">{description}</p>
          )}
        </div>
      )}
      <div className="settings-card overflow-visible rounded-[16px] border border-line bg-card shadow-[0_1px_2px_rgba(0,0,0,0.04)]">
        <div className="settings-rows">{children}</div>
      </div>
    </section>
  );
};
