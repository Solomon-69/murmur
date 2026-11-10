import React from "react";

/** Stand-in for SettingsGroup that renders only its rows (no title or card). */
export const BareGroup: React.FC<{
  title?: string;
  children: React.ReactNode;
}> = ({ children }) => <>{children}</>;
